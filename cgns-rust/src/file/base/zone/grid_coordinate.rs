//! Based on: <https://cgns.github.io/CGNS_docs_current/midlevel/grid.html>

pub mod coords;

use std::ffi;

use anyhow::{anyhow, Result};
use cgns_sys::*;

use self::coords::Coordinates;
use crate::traits::{CGNSNode, CGNSParent};
use crate::utils::{ier_cg_fn, CGNSError, CGIO_NAME_BUFFER_LENGTH};

use super::Zone;

#[derive(Debug, Clone)]
/// CGNS node `GridCoordinates_t`
pub struct GridCoordinates<'a> {
    pub name: String,
    pub bounding_box: Option<[f64; 3]>,
    id: i32,
    zone: &'a Zone<'a>,
}

impl<'a> CGNSNode<'a> for GridCoordinates<'a> {
    type Parent = Zone<'a>;

    fn from_id(parent: &'a Self::Parent, id: i32) -> Result<Self> {
        const DEFAULT_BBOX: [f64; 3] = [-1.; 3];
        let mut grid_name = [0i8; CGIO_NAME_BUFFER_LENGTH];
        let mut bounding_box = DEFAULT_BBOX;
        let dtype = DataType_t::RealDouble;

        ier_cg_fn!(cg_grid_read(
            parent.base.file.id(),
            parent.base.id(),
            parent.id(),
            id,
            grid_name.as_mut_ptr().cast(),
        ))?;
        ier_cg_fn!(cg_grid_bounding_box_read(
            parent.base.file.id(),
            parent.base.id(),
            parent.id(),
            id,
            dtype,
            bounding_box.as_mut_ptr() as *mut ffi::c_void,
        ))?;
        let name = unsafe { ffi::CStr::from_ptr(grid_name.as_ptr()) }
            .to_str()
            .unwrap()
            .to_owned();
        // If the bounding box is not set, CGNS will print a warning to stdout but won't return an error.
        let bounding_box = if bounding_box == DEFAULT_BBOX {
            None
        } else {
            Some(bounding_box)
        };

        Ok(GridCoordinates {
            name,
            bounding_box,
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

impl<'a> CGNSParent<'a, Coordinates<'a>> for GridCoordinates<'a> {
    fn num_child(&self) -> Result<i32> {
        if self.id > 1 {
            return Err(anyhow!(
                "Can only read one GridCoordinates_t node. Use cgns-sys to read more."
            ));
        }
        let mut number = 0;
        ier_cg_fn!(cg_ncoords(
            self.zone.base.file.id(),
            self.zone.base.id(),
            self.zone.id(),
            &mut number
        ))?;
        Ok(number)
    }
}
