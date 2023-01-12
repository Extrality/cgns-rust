//! Module dedicated to sections, which hold element connectivity.
//!
//! Based on: <https://cgns.github.io/CGNS_docs_current/midlevel/grid.html#elements>

use anyhow::{anyhow, Context};
use cgns_sys::*;

use super::Zone;
use crate::traits::CGNSNode;
use crate::utils::{bytes2string, ier_cg_fn, Result, CGIO_NAME_BUFFER_LENGTH};

/// Get point per face of an element type
#[inline]
pub fn npe(elem_id: u32) -> Result<i64> {
    let elem_type = unsafe { std::mem::transmute(elem_id) };
    let mut npe = 0;
    ier_cg_fn!(cg_npe(elem_type, &mut npe))?;
    Ok(npe as i64)
}

#[derive(Debug, Clone)]
/// CGNS node `Elements_t`
pub struct Section<'a> {
    pub name: String,
    /// Type of enclosed elements.
    pub elem_type: ElementType_t,
    /// Index of first element in the section.
    pub elem_start: i64,
    /// Index of last element in the section.
    pub elem_end: i64,
    /// "Index of last boundary element in the section. Set to zero if the elements are unsorted."
    pub nbndry: i32, // ???
    /// For boundary or interface elements, the parent_data array contains information on the cell(s) and cell face(s) sharing the element.
    pub has_parent_data: bool,
    id: i32,
    pub zone: &'a Zone<'a>,
}

#[derive(Debug, Clone)]
pub struct Elements {
    pub connectivity: Vec<i64>,
    pub offsets: Option<Vec<i64>>,
}

/// Special fix for CGNS `3.4.1`.
/// Converts CGNS 3.3 style connectivity to CGNS 4 style (<https://cgns.github.io/ProposedExtensions/NGON-CPEX-0041-v0.16.pdf>).
///
/// Returns the new length of `connectivity`.
fn fix_missing_elements_offsets(
    connectivity: &mut [i64],
    offsets: &mut [i64],
    elem_type: ElementType_t,
) -> Result<usize> {
    offsets[0] = 0;

    let connectivity_len = match elem_type {
        ElementType_t::NFACE_n | ElementType_t::NGON_n => {
            let mut idx_connect_new = 0;
            let mut idx_connect_old = 0;
            for idx_elem in 0..offsets.len() - 1 {
                let elem_size = connectivity[idx_connect_old];
                idx_connect_old += 1;
                offsets[idx_elem + 1] = offsets[idx_elem] + elem_size;
                for _ in 0..elem_size {
                    connectivity[idx_connect_new] = connectivity[idx_connect_old];
                    idx_connect_new += 1;
                    idx_connect_old += 1;
                }
            }
            idx_connect_new
        }
        ElementType_t::MIXED => {
            let mut idx_connect = 0;
            for idx_elem in 0..offsets.len() - 1 {
                let elem_size = npe(connectivity[idx_connect] as u32)?;
                idx_connect += elem_size as usize + 1;
                offsets[idx_elem + 1] = idx_connect as i64;
            }
            idx_connect
        }
        _ => {
            return Err(anyhow!(
                "Invalid elem type for missing connectivity offset: {:?}",
                elem_type
            )
            .into())
        }
    };

    Ok(connectivity_len)
}

impl<'a> Section<'a> {
    /// Whether the element connectivity is composed of `connectivity` and `offsets` or not.
    #[inline]
    pub fn elements_have_offsets(&self) -> bool {
        matches!(
            self.elem_type,
            ElementType_t::MIXED | ElementType_t::NGON_n | ElementType_t::NFACE_n
        )
    }

    /// A method to read connectivity values directly to a buffer, to avoid copying large amount of data.
    /// See [`Self::read_elements()`].
    ///
    /// Because of an issue in the CGNS lib (caused by CGNS 3.4.1),
    /// the length of the connectivity might change and is returned by this function.
    pub fn read_elements_to_buff(
        &self,
        connectivity: &mut [i64],
        mut offsets: Option<&mut [i64]>,
    ) -> Result<usize> {
        const CONTROL_PATTERN: [i64; 2] = [1337, 420];
        if self.elements_have_offsets() {
            if let Some(offsets) = &offsets {
                if offsets.len() != self.offsets_len() as usize {
                    return Err(anyhow!(
                        "Offset buffer is of len {} but should be {}",
                        offsets.len(),
                        self.size()
                    )
                    .into());
                }
            } else {
                return Err(anyhow!("Offset buffer is required but is None").into());
            }
        }
        if connectivity.len() != self.data_size()? as usize {
            return Err(anyhow!(
                "Connectivity buffer is of len {} but should be {}",
                connectivity.len(),
                self.data_size()?
            )
            .into());
        }

        let offset_ptr = if let Some(offsets) = &mut offsets {
            let check_len = 2.min(offsets.len());
            offsets[..check_len].copy_from_slice(&CONTROL_PATTERN[..check_len]);
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
        let mut final_connectivity_size = connectivity.len();
        if let Some(offsets) = offsets {
            if offsets.starts_with(&CONTROL_PATTERN[..2.min(offsets.len())]) {
                final_connectivity_size =
                    fix_missing_elements_offsets(connectivity, offsets, self.elem_type)
                        .context("Could not rebuild missing offsets array")?;
            }
        }

        Ok(final_connectivity_size)
    }

    /// Read the element connectivity
    pub fn read_elements(&self) -> Result<Elements> {
        let has_offsets = self.elements_have_offsets();
        let mut connectivity = vec![0; self.data_size()? as usize];
        let mut offsets = if has_offsets {
            Some(vec![0; self.offsets_len() as usize])
        } else {
            None
        };
        let conn_len = self.read_elements_to_buff(&mut connectivity, offsets.as_deref_mut())?;
        connectivity.truncate(conn_len);

        Ok(Elements {
            connectivity,
            offsets,
        })
    }

    /// Get point per face of an element type.
    #[inline]
    pub fn npe(&self) -> Result<i64> {
        npe(self.elem_type as u32)
    }

    /// Number of elements in the section.
    #[inline]
    pub fn size(&self) -> i64 {
        // +1 because CGNS arrays start at one
        self.elem_end - self.elem_start + 1
    }

    /// Elements offsets length ([`Self::size()`] + 1).
    #[inline]
    pub fn offsets_len(&self) -> i64 {
        self.size() + 1
    }

    /// Element connectivity length.
    #[inline]
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

impl<'a> CGNSNode<'a> for Section<'a> {
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

        Ok(Section {
            name,
            elem_type,
            elem_start: start,
            elem_end: end,
            has_parent_data: is_parent_defined == 1,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_missing_elements_offsets() {
        let mut connectivity = [3, 1, 2, 3, 4, 3, 2, 1, 4, 4, 4, 3, 2, 1, 2, 1, 2];
        let mut offsets = [0, 0, 0, 0, 0];
        let new_conn_len =
            fix_missing_elements_offsets(&mut connectivity, &mut offsets, ElementType_t::NGON_n)
                .unwrap();
        assert_eq!(new_conn_len, 13);
        assert_eq!(
            &connectivity[..new_conn_len],
            &[1, 2, 3, 3, 2, 1, 4, 4, 3, 2, 1, 1, 2]
        );
        assert_eq!(offsets, [0, 3, 7, 11, 13]);
    }
}
