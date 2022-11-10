//! Based on: <https://cgns.github.io/CGNS_docs_current/midlevel/grid.html>

use std::ffi;

use anyhow::{anyhow, Result};
use cgns_sys::*;

use crate::traits::CGNSNode;
use crate::utils::{ier_cg_fn, CGNSError, CGIO_NAME_BUFFER_LENGTH};

use super::Zone;

#[derive(Debug, Clone)]
/// CGNS node `Elements_t`
pub struct Element<'a> {
    pub name: String,
    pub elem_type: ElementType_t,
    pub range_start: i64,
    pub range_end: i64,
    nbndry: i32, // ???
    is_parent_data_defined: bool,
    id: i32,
    pub zone: &'a Zone<'a>,
}

#[derive(Debug, Clone)]
pub enum Connectivity {
    OldNGON(InlineNConnectivity),
    OldNFACE(InlineNConnectivity),
    OldMixed(InlineNConnectivity),

    NGON(NConnectivity),
    NFACE(NConnectivity),
    MIXED(NConnectivity),
}

/// Support for older NGON, NFACE, MIXED element connectivity.
/// <https://cgns.github.io/ProposedExtensions/NGON-CPEX-0041-v0.16.pdf>
#[derive(Debug, Clone)]
pub struct InlineNConnectivity(Vec<i64>);

#[derive(Debug, Clone)]
pub struct NConnectivity {
    connectivity: Vec<i64>,
    offsets: Vec<i64>,
}

impl Connectivity {
    pub fn to_vtk(&self) {
        match self {
            Self::OldNGON(_) => (),
            _ => (),
        };

        // cells (inline connectivity)
        // cell_type
        // points
    }
}

// #[derive(Debug, Clone)]
// struct StaticConnectivity {
//     element_type: ElementType_t,
//     connectivity: Vec<i64>
// }

impl<'a> Element<'a> {
    pub fn read(&self) -> Result<Connectivity> {
        let old_cgns_compat = self.zone.base.file.version < 4.;

        println!("file version: {}", self.zone.base.file.version);

        // Ok(Connectivity::NFACE(NConnectivity{connectivity: Vec::new(), offsets: Vec::new()}))
        match (self.elem_type, old_cgns_compat) {
            (ElementType_t::MIXED, false) => Ok(Connectivity::MIXED(self.read_nconnectivity()?)),
            (ElementType_t::NGON_n, false) => Ok(Connectivity::NGON(self.read_nconnectivity()?)),
            (ElementType_t::NFACE_n, false) => Ok(Connectivity::NFACE(self.read_nconnectivity()?)),
            (ElementType_t::NGON_n, true) => {
                Ok(Connectivity::OldNGON(self.read_inline_nconnectivity()?))
            }
            (ElementType_t::NFACE_n, true) => {
                Ok(Connectivity::OldNFACE(self.read_inline_nconnectivity()?))
            }
            (ElementType_t::MIXED, true) => {
                Ok(Connectivity::OldMixed(self.read_inline_nconnectivity()?))
            }
            (ElementType_t::ElementTypeNull | ElementType_t::ElementTypeUserDefined, _) => {
                anyhow::bail!("Invalid element type")
            }
            _ => anyhow::bail!("Unsupported element type"),
        }
    }

    fn read_inline_nconnectivity(&self) -> Result<InlineNConnectivity> {
        let mut connectivity = vec![0; self.data_size().unwrap() as usize];
        // Always use poly_elements_read for eg support for old NGON/NFACE
        ier_cg_fn!(cg_poly_elements_read(
            self.zone.base.file.id(),
            self.zone.base.id(),
            self.zone.id(),
            self.id,
            connectivity.as_mut_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut()
        ))?;

        Ok(InlineNConnectivity(connectivity))
    }

    fn read_nconnectivity(&self) -> Result<NConnectivity> {
        let mut connectivity = vec![0; self.data_size()? as usize];
        let mut offsets = vec![0; self.element_size() as usize + 1];

        ier_cg_fn!(cg_poly_elements_read(
            self.zone.base.file.id(),
            self.zone.base.id(),
            self.zone.id(),
            self.id,
            connectivity.as_mut_ptr(),
            offsets.as_mut_ptr(),
            std::ptr::null_mut()
        ))?;

        // https://cgnsorg.atlassian.net/browse/CGNS-285
        debug_assert_ne!(
            offsets,
            vec![0; self.element_size() as usize + 1],
            "Missing offsets ?!"
        );

        Ok(NConnectivity {
            connectivity,
            offsets,
        })
    }

    /// Get point per face of an element type
    fn npe(&self) -> Result<i64> {
        let mut npe = 0;
        ier_cg_fn!(cg_npe(self.elem_type, &mut npe))?;
        Ok(npe as i64)
    }

    ///
    fn element_size(&self) -> i64 {
        self.range_end - self.range_start + 1
    }

    fn data_size(&self) -> Result<i64> {
        let mut data_size = 0;
        ier_cg_fn!(cg_ElementDataSize(
            self.zone.base.file.id(),
            self.zone.base.id(),
            self.zone.id(),
            self.id(),
            &mut data_size
        ))?;
        Ok(data_size)
        // Ok(self.element_size() * self.npe()?)
    }
}

// impl<'a> Read<'a, f32> for Element<'a> {
//     fn read(&self) -> Result<Vec<f32>> {
//         let mut elements = Box::new([0i64; 100]);
//         let mut connect_offset = 0;

//         ier_cg_fn!(cg_poly_elements_read(
//             self.zone.base.file.id(),
//             self.zone.base.id(),
//             self.zone.id(),
//             self.id,
//             elements.as_mut_ptr(),
//             &mut connect_offset,
//             std::ptr::null_mut() // parent_data
//         ))?;

//     }
// }

impl<'a> CGNSNode<'a> for Element<'a> {
    type Parent = Zone<'a>;

    fn from_id(parent: &'a Self::Parent, id: i32) -> Result<Self> {
        let mut elem_name = [0u8; CGIO_NAME_BUFFER_LENGTH];
        let mut elem_type = ElementType_t::ElementTypeNull;
        let mut start = 0;
        let mut end = 0;
        let mut nbndry = 0;
        let mut is_parent_defined = 0;

        ier_cg_fn!(cg_section_read(
            parent.base.file.id(),
            parent.base.id(),
            parent.id(),
            id,
            elem_name.as_mut_ptr().cast(),
            &mut elem_type,
            &mut start,
            &mut end,
            &mut nbndry,
            &mut is_parent_defined
        ))?;

        let name = unsafe { ffi::CStr::from_ptr(elem_name.as_ptr().cast()) }
            .to_str()
            .unwrap()
            .to_owned();

        Ok(Element {
            name,
            elem_type,
            range_start: start,
            range_end: end,
            is_parent_data_defined: is_parent_defined == 1,
            nbndry,
            id,
            zone: parent,
        })
    }
    fn id(&self) -> i32 {
        self.id
    }
    fn parent(&self) -> &Self::Parent {
        self.zone
    }
}
