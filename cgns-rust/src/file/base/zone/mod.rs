//! Based on: <https://cgns.github.io/CGNS_docs_current/midlevel/structural.html#zone>

pub mod arbitrary_grid_motion;
pub mod boundary_conditions;
pub mod flow_solution;
pub mod grid_coordinate;
pub mod rigid_grid_motion;
pub mod section;
pub mod zone_grid_connectivity;

use cgns_sys::*;

use self::boundary_conditions::BC;
use self::flow_solution::FlowSolution;
use self::grid_coordinate::GridCoordinates;
use self::section::Section;
use super::Base;
use crate::utils::{ier_cg_fn, string2bytes, CGIO_NAME_BUFFER_LENGTH};
use crate::{
    traits::{CGNSNode, CGNSNodeIterator, CGNSParent},
    utils::{bytes2string, Result},
};

#[derive(Debug, Clone, PartialEq)]
/// CGNS node [`Zone_t`](https://cgns.github.io/CGNS_docs_current/sids/cgnsbase.html#Zone)
pub struct Zone<'a> {
    pub name: String,
    raw_size: [i64; 9],
    ztype: ZoneType_t,
    pub base: &'a Base<'a>,
    id: i32,
}

/// TODO: lots to improve here
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneSize<'a> {
    pub vertices: &'a [i64],
    /// For structured zones: `number_of_cells = number_of_vertices - 1`
    pub cells: &'a [i64],
    /// Always 0 for structured grids
    pub bound_vertices: &'a [i64],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneSize1D {
    pub vertices: i64,
    pub cells: i64,
    /// Always 0 for structured grids
    pub bound_vertices: i64,
}

impl<'a> ZoneSize<'a> {
    pub fn raw(&self) -> [i64; 9] {
        let mut res = [0; 9];
        let mut it = res.iter_mut();
        for arr in [self.vertices, self.cells, self.bound_vertices] {
            for val in arr {
                *it.next().unwrap() = *val;
            }
        }
        res
    }
    pub fn total_size(&self) -> ZoneSize1D {
        ZoneSize1D {
            vertices: self.vertices.iter().sum(),
            cells: self.cells.iter().sum(),
            bound_vertices: self.bound_vertices.iter().sum(),
        }
    }
}

impl<'a> Zone<'a> {
    pub fn iter_solutions(&'a self) -> Result<CGNSNodeIterator<'a, FlowSolution<'a>>> {
        self.iter()
    }

    pub fn iter_sections(&'a self) -> Result<CGNSNodeIterator<'a, Section<'a>>> {
        self.iter()
    }

    pub fn iter_grid_coordinates(&'a self) -> Result<CGNSNodeIterator<'a, GridCoordinates<'a>>> {
        self.iter()
    }

    pub fn iter_boundary_conditions(&'a self) -> Result<CGNSNodeIterator<'a, BC<'a>>> {
        self.iter()
    }

    pub fn write(
        base: &'a Base,
        name: String,
        raw_size: [i64; 9],
        ztype: ZoneType_t,
    ) -> Result<Self> {
        let c_name = string2bytes(&name)?;
        let mut id = 0;
        ier_cg_fn!(cg_zone_write(
            base.file.id,
            base.id,
            c_name.as_ptr(),
            raw_size.as_ptr(),
            ztype,
            &mut id
        ))?;
        Ok(Self {
            name,
            raw_size,
            ztype,
            base,
            id,
        })
    }

    pub fn size(&self) -> ZoneSize {
        match (self.ztype, self.base.phys_dim) {
            (ZoneType_t::Structured, 1) => ZoneSize {
                vertices: &self.raw_size[0..1],
                cells: &self.raw_size[1..2],
                bound_vertices: &self.raw_size[2..3],
            },
            (ZoneType_t::Structured, 2) => ZoneSize {
                vertices: &self.raw_size[0..2],
                cells: &self.raw_size[2..4],
                bound_vertices: &self.raw_size[4..6],
            },
            (ZoneType_t::Structured, 3) => ZoneSize {
                vertices: &self.raw_size[0..3],
                cells: &self.raw_size[3..6],
                bound_vertices: &self.raw_size[6..9],
            },
            (ZoneType_t::Unstructured, _) => ZoneSize {
                vertices: &self.raw_size[0..1],
                cells: &self.raw_size[1..2],
                bound_vertices: &self.raw_size[2..3],
            },
            z @ _ => {
                println!("Invalid zone type or size: {:?}", z); // TODO
                ZoneSize {
                    vertices: &[],
                    cells: &[],
                    bound_vertices: &[],
                }
            }
        }
    }

    pub fn total_size(&self) -> ZoneSize1D {
        self.size().total_size()
    }
}

impl<'a> CGNSNode<'a> for Zone<'a> {
    type Parent = Base<'a>;

    fn from_id(parent: &'a Self::Parent, id: i32) -> Result<Self> {
        let mut raw_size = [0; 9];
        let mut name_raw = [0u8; CGIO_NAME_BUFFER_LENGTH];
        let mut ztype = ZoneType_t::ZoneTypeNull;

        ier_cg_fn!(cg_zone_read(
            parent.file.id(),
            parent.id(),
            id,
            name_raw.as_mut_ptr().cast(),
            raw_size.as_mut_ptr()
        ))?;

        ier_cg_fn!(cg_zone_type(parent.file.id(), parent.id(), id, &mut ztype))?;

        let name = bytes2string(&name_raw)?;

        Ok(Zone {
            name,
            raw_size,
            ztype,
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

impl<'a> CGNSParent<'a, Section<'a>> for Zone<'a> {
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

impl<'a> CGNSParent<'a, BC<'a>> for Zone<'a> {
    fn num_child(&self) -> Result<i32> {
        let mut number = 0;
        ier_cg_fn!(cg_nbocos(
            self.base.file.id(),
            self.base.id(),
            self.id,
            &mut number
        ))?;
        Ok(number)
    }
}

#[cfg(test)]
mod tests {
    use testdir::testdir;

    use super::*;
    use crate::{file::tests::cgns_file, library::LibraryHandle};

    #[test]
    fn can_write_zone() {
        let name_1 = "Stratovarius".to_string();
        let name_2 = "ElementEighty".to_string();
        let size1 = ZoneSize {
            vertices: &[9999],
            cells: &[999],
            bound_vertices: &[123],
        };
        let size2 = ZoneSize {
            vertices: &[12, 12, 12],
            cells: &[11, 11, 11],
            bound_vertices: &[0, 0, 0],
        };
        let lib = LibraryHandle::acquire();

        // 1. Can I write zones ?
        let (_p, f) = cgns_file(&lib, testdir!(), 0);
        let b = Base::write(&f, "ArcticBase".to_string(), 3, 3).unwrap();
        let z1 = Zone::write(&b, name_1.clone(), [1; 9], ZoneType_t::Unstructured).unwrap();
        let z2 = Zone::write(&b, name_2, size1.raw(), ZoneType_t::Unstructured).unwrap();

        assert_eq!(z1.id, 1);
        assert_eq!(z1.name, name_1);
        assert_eq!(z2.id, 2);
        assert_eq!(z2.raw_size, [9999, 999, 123, 0, 0, 0, 0, 0, 0]);

        // 2. Can I overwrite and read them ?
        let z1_b = Zone::write(&b, name_1, size2.raw(), ZoneType_t::Structured).unwrap();

        let zones: Vec<_> = b.iter_zones().unwrap().collect();
        assert_eq!(zones.as_slice(), &[z1_b, z2]);
        f.close().unwrap();
    }
}
