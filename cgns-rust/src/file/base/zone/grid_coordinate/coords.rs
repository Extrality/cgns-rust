//! Based on: <https://cgns.github.io/CGNS_docs_current/midlevel/grid.html#gridcoordinates>

use std::ffi;

use cgns_sys::*;

use super::GridCoordinates;
use crate::traits::{CGNSNode, Read};
use crate::utils::{bytes2string, ier_cg_fn, Result, CGIO_NAME_BUFFER_LENGTH};

#[derive(Debug, Clone)]
/// CGNS node `DataArray_t` under a `GridCoordinates_t`
pub struct Coordinates<'a> {
    pub name: String,
    pub dtype: DataType_t,
    grid_coordinate: &'a GridCoordinates<'a>,
    id: i32,
}

impl<'a> Read<'a, f32> for Coordinates<'a> {
    fn read(&self) -> Result<Vec<f32>> {
        let one = 1;
        let nb_points = self.grid_coordinate.zone.total_size().vertices;
        let cname = ffi::CString::new(self.name.as_bytes()).unwrap();
        let mut data = vec![0f32; nb_points as usize];

        ier_cg_fn!(cg_coord_read(
            self.grid_coordinate.zone.base.file.id(),
            self.grid_coordinate.zone.base.id(),
            self.grid_coordinate.zone.id(),
            cname.as_ptr(),
            DataType_t::RealSingle,
            &one,
            &nb_points,
            data.as_mut_ptr().cast()
        ))?;

        Ok(data)
    }
}

impl<'a> CGNSNode<'a> for Coordinates<'a> {
    type Parent = GridCoordinates<'a>;

    fn from_id(parent: &'a Self::Parent, id: i32) -> Result<Self> {
        let mut coord_name = [0u8; CGIO_NAME_BUFFER_LENGTH];
        let mut dtype = DataType_t::DataTypeNull;

        ier_cg_fn!(cg_coord_info(
            parent.zone.base.file.id(),
            parent.zone.base.id(),
            parent.zone.id(),
            id,
            &mut dtype,
            coord_name.as_mut_ptr().cast(),
        ))?;
        let name = bytes2string(&coord_name)?;

        Ok(Coordinates {
            name,
            dtype,
            id,
            grid_coordinate: parent,
        })
    }
    fn id(&self) -> i32 {
        self.id
    }
    fn parent(&self) -> &Self::Parent {
        self.grid_coordinate
    }
}
