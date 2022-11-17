//! Based on: <https://cgns.github.io/CGNS_docs_current/midlevel/grid.html>

use std::ffi;

use anyhow::{anyhow, Result};
use cgns_sys::*;

use crate::traits::CGNSNode;
use crate::utils::{bytes2string, ier_cg_fn, CGNSError, CGIO_NAME_BUFFER_LENGTH};

use super::Zone;

/// Get point per face of an element type
pub fn npe(elem_id: u32) -> Result<i64> {
    let elem_type = unsafe { std::mem::transmute(elem_id) };
    let mut npe = 0;
    ier_cg_fn!(cg_npe(elem_type, &mut npe))?;
    Ok(npe as i64)
}

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
pub struct Connectivity {
    pub connectivity: Vec<i64>,
    pub offsets: Option<Vec<i64>>,
}

impl<'a> Element<'a> {
    /// Whether the connectivity is composed of `connectivity` and `offsets` or not
    pub fn connectivity_has_offsets(&self) -> bool {
        let old_cgns_compat = self.zone.base.file.version < 4.;
        !old_cgns_compat
            && matches!(
                self.elem_type,
                ElementType_t::MIXED | ElementType_t::NGON_n | ElementType_t::NFACE_n
            )
    }

    /// A method to read connectivity values directly to a buffer, to avoid copying large amount of data
    /// See [`read_connectivity()`]
    pub fn read_connectivity_to_buff(
        &self,
        connectivity: &mut [i64],
        offsets: Option<&mut [i64]>,
    ) -> Result<()> {
        if self.connectivity_has_offsets() {
            if let Some(offsets) = &offsets {
                if offsets.len() != self.size() as usize {
                    anyhow::bail!(
                        "Offset buffer is of len {} but should be {}",
                        offsets.len(),
                        self.size()
                    );
                }
            } else {
                anyhow::bail!("Offset buffer is required but is None");
            }
        }
        if connectivity.len() != self.data_size()? as usize {
            anyhow::bail!(
                "Connectivity buffer is of len {} but should be {}",
                connectivity.len(),
                self.data_size()?
            );
        }

        let offset_ptr = if let Some(offsets) = offsets {
            offsets.as_mut_ptr()
        } else {
            std::ptr::null_mut()
        };

        #[allow(unused_unsafe)]
        ier_cg_fn!(cg_poly_elements_read(
            self.zone.base.file.id(),
            self.zone.base.id(),
            self.zone.id(),
            self.id,
            connectivity.as_mut_ptr(),
            offset_ptr,
            std::ptr::null_mut()
        ))?;

        Ok(())
    }

    /// Read the element connectivity
    pub fn read_connectivity(&self) -> Result<Connectivity> {
        let has_offsets = self.connectivity_has_offsets();
        let mut connectivity = vec![0; self.data_size()? as usize];
        let mut offsets = if has_offsets {
            Some(vec![0; self.size() as usize + 1])
        } else {
            None
        };

        let offset_ptr = if let Some(offsets) = &mut offsets {
            offsets.as_mut_ptr()
        } else {
            std::ptr::null_mut()
        };

        ier_cg_fn!(cg_poly_elements_read(
            self.zone.base.file.id(),
            self.zone.base.id(),
            self.zone.id(),
            self.id,
            connectivity.as_mut_ptr(),
            offset_ptr,
            std::ptr::null_mut()
        ))?;

        Ok(Connectivity {
            connectivity,
            offsets,
        })
    }

    /// Get point per face of an element type
    pub fn npe(&self) -> Result<i64> {
        let mut npe = 0;
        ier_cg_fn!(cg_npe(self.elem_type, &mut npe))?;
        Ok(npe as i64)
    }

    pub fn size(&self) -> i64 {
        self.range_end - self.range_start + 1
    }

    pub fn data_size(&self) -> Result<i64> {
        let mut data_size = 0;
        ier_cg_fn!(cg_ElementDataSize(
            self.zone.base.file.id(),
            self.zone.base.id(),
            self.zone.id(),
            self.id(),
            &mut data_size
        ))?;
        Ok(data_size)
    }
}

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
        let name = bytes2string(&elem_name)?;

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
