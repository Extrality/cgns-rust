//! Based on: <https://cgns.github.io/CGNS_docs_current/midlevel/solution.html#flowsolution_array>

use core::panic;
use std::ffi;

use anyhow::{anyhow, Result};
use cgns_sys::*;
use heapless;

use super::FlowSolution;
use crate::traits::{CGNSNode, Read};
use crate::utils::bytes2string;
use crate::{
    file::base::zone::ZoneSize,
    utils::{ier_cg_fn, CGNSError, CGIO_NAME_BUFFER_LENGTH},
};

#[derive(Debug, Clone)]
/// CGNS node `DataArray_t` under a `FlowSolution_t`
pub struct Field<'a> {
    pub name: String,
    pub datatype: DataType_t,
    id: i32,
    flow_solution: &'a FlowSolution<'a>,
}

impl<'a> Read<'a, f32> for Field<'a> {
    fn read(&self) -> Result<Vec<f32>> {
        let range_min = [1, 1, 1]; // TODO ? Rind ?
        let range_max = self.shape();
        let len = range_max.iter().product::<i64>() as usize;
        let mut data = vec![0f32; len];
        let name = ffi::CString::new(self.name.as_bytes()).unwrap();

        ier_cg_fn!(cg_field_read(
            self.flow_solution.zone.base.file.id(),
            self.flow_solution.zone.base.id(),
            self.flow_solution.zone.id(),
            self.flow_solution.id(),
            name.as_ptr(),
            DataType_t::RealSingle,
            range_min.as_ptr(),
            range_max.as_ptr(),
            data.as_mut_ptr().cast()
        ))?;

        Ok(data)
    }
}

impl<'a> CGNSNode<'a> for Field<'a> {
    type Parent = FlowSolution<'a>;

    fn from_id(parent: &'a Self::Parent, id: i32) -> Result<Self> {
        let mut name = [0u8; CGIO_NAME_BUFFER_LENGTH];
        let mut datatype = DataType_t::DataTypeNull;

        ier_cg_fn!(cg_field_info(
            parent.zone.base.file.id(),
            parent.zone.base.id(),
            parent.zone.id(),
            parent.id(),
            id,
            &mut datatype,
            name.as_mut_ptr().cast()
        ))?;
        let name = bytes2string(&name)?;

        Ok(Field {
            name,
            datatype,
            id,
            flow_solution: parent,
        })
    }

    fn id(&self) -> i32 {
        self.id
    }

    fn parent(&self) -> &Self::Parent {
        self.flow_solution
    }
}

impl<'a> Field<'a> {
    pub fn location(&self) -> &GridLocation_t {
        &self.flow_solution.grid_location
    }

    pub fn shape(&self) -> heapless::Vec<i64, 3> {
        let mut range_max = heapless::Vec::new();
        let zone_size = &self.flow_solution.zone.size;
        let grid_location = self.location();
        match (zone_size, grid_location) {
            (ZoneSize::Unstructured(zu), GridLocation_t::Vertex) => {
                range_max.push(zu.n_vertex).unwrap();
            }
            (ZoneSize::Unstructured(zu), GridLocation_t::CellCenter) => {
                range_max.push(zu.n_cells).unwrap();
            }
            (ZoneSize::Structured2D(zs2), GridLocation_t::Vertex) => {
                range_max.copy_from_slice(&zs2.n_vertex);
            }
            (ZoneSize::Structured2D(zs2), GridLocation_t::CellCenter) => {
                range_max.copy_from_slice(&zs2.n_cells);
            }
            (ZoneSize::Structured3D(zs3), GridLocation_t::Vertex) => {
                range_max.copy_from_slice(&zs3.n_vertex);
            }
            (ZoneSize::Structured3D(zs3), GridLocation_t::CellCenter) => {
                range_max.copy_from_slice(&zs3.n_cells);
            }
            _ => panic!(
                "Case not handled (yet): zone_size: {:?} grid_location: {:?}",
                zone_size, grid_location
            ), // TODO !
        }
        range_max
    }
}
