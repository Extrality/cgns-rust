pub mod arbitrary_grid_motion;
pub mod elements;
pub mod flow_solution;
pub mod grid_coordinate;
pub mod rigid_grid_motion;
pub mod zone_grid_connectivity;

use std::ffi;

use anyhow::{anyhow, Result};
use cgns_sys::*;

use self::elements::Element;
use self::flow_solution::FlowSolution;
use self::grid_coordinate::GridCoordinates;
use crate::traits::{CGNSNode, CGNSNodeIterator, CGNSParent};

use super::Base;
use crate::utils::{ier_cg_fn, CGNSError, CGIO_NAME_BUFFER_LENGTH};

#[derive(Debug, Clone, PartialEq)]
/// CGNS node `Zone_t`
pub struct Zone<'a> {
    pub name: String,
    pub size: ZoneSize,
    ztype: ZoneType_t,
    pub index_dimension: i32,
    pub base: &'a Base<'a>,
    id: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZoneSize {
    Structured3D(ZoneSizeStructured<3>),
    Structured2D(ZoneSizeStructured<2>),
    Unstructured(ZoneSizeUnstructured),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSizeStructured<const DIMENSIONS: usize> {
    pub n_vertex: [i64; DIMENSIONS],
    pub n_cells: [i64; DIMENSIONS],
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSizeUnstructured {
    pub n_vertex: i64,
    pub n_cells: i64,
    pub n_bound_vertex: i64,
}

impl<'a> Zone<'a> {
    pub fn iter_solutions(&'a self) -> Result<CGNSNodeIterator<'a, FlowSolution<'a>>> {
        self.iter()
    }

    pub fn iter_elements(&'a self) -> Result<CGNSNodeIterator<'a, Element<'a>>> {
        self.iter()
    }

    pub fn iter_grid_coordinates(&'a self) -> Result<CGNSNodeIterator<'a, GridCoordinates<'a>>> {
        self.iter()
    }
}

impl ZoneSize {
    fn from_raw_values(vals: [i64; 9], zone_type: ZoneType_t, phys_dim: i32) -> Result<Self> {
        match (zone_type, phys_dim) {
            (ZoneType_t::Structured, 2) => Ok(ZoneSize::Structured2D(ZoneSizeStructured {
                n_vertex: [vals[0], vals[1]],
                n_cells: [vals[2], vals[3]],
            })),
            (ZoneType_t::Structured, 3) => Ok(ZoneSize::Structured3D(ZoneSizeStructured {
                n_vertex: [vals[0], vals[1], vals[2]],
                n_cells: [vals[3], vals[4], vals[5]],
            })),
            (ZoneType_t::Unstructured, _) => Ok(ZoneSize::Unstructured(ZoneSizeUnstructured {
                n_vertex: vals[0],
                n_cells: vals[1],
                n_bound_vertex: vals[2],
            })),
            _ => Err(anyhow!(
                "Cannot handle zone_type {:?} with physical dimensions {}",
                zone_type,
                phys_dim
            )),
        }
    }
}

impl<'a> CGNSNode<'a> for Zone<'a> {
    type Parent = Base<'a>;

    fn from_id(parent: &'a Self::Parent, id: i32) -> Result<Self> {
        let mut size = [0; 9];
        let mut name_raw = [0 as ffi::c_char; CGIO_NAME_BUFFER_LENGTH];
        let mut ztype = ZoneType_t::ZoneTypeNull;
        let mut index_dimension = 0;

        ier_cg_fn!(cg_zone_read(
            parent.file.id(),
            parent.id(),
            id,
            name_raw.as_mut_ptr(),
            size.as_mut_ptr()
        ))?;

        ier_cg_fn!(cg_zone_type(parent.file.id(), parent.id(), id, &mut ztype))?;
        ier_cg_fn!(cg_index_dim(
            parent.file.id(),
            parent.id(),
            id,
            &mut index_dimension
        ))?;

        let size = ZoneSize::from_raw_values(size, ztype, parent.phys_dim)?;

        Ok(Zone {
            name: unsafe { ffi::CStr::from_ptr(name_raw.as_ptr()) }
                .to_str()?
                .to_string(),
            size,
            ztype,
            index_dimension,
            base: parent,
            id,
        })
    }
    fn id(&self) -> i32 {
        self.id
    }
    fn parent(&self) -> &Self::Parent {
        self.base
    }
}

impl<'a> CGNSParent<'a, FlowSolution<'a>> for Zone<'a> {
    fn num_child(&self) -> Result<i32> {
        let mut number = 0;
        ier_cg_fn!(cg_nsols(
            self.base.file.id(),
            self.base.id(),
            self.id,
            &mut number
        ))?;
        Ok(number)
    }
}

impl<'a> CGNSParent<'a, GridCoordinates<'a>> for Zone<'a> {
    fn num_child(&self) -> Result<i32> {
        let mut number = 0;
        ier_cg_fn!(cg_ngrids(
            self.base.file.id(),
            self.base.id(),
            self.id,
            &mut number
        ))?;
        Ok(number)
    }
}

impl<'a> CGNSParent<'a, Element<'a>> for Zone<'a> {
    fn num_child(&self) -> Result<i32> {
        let mut number = 0;
        ier_cg_fn!(cg_nsections(
            self.base.file.id(),
            self.base.id(),
            self.id,
            &mut number
        ))?;
        Ok(number)
    }
}
