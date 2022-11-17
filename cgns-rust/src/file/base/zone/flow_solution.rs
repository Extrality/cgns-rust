//! Based on: <https://cgns.github.io/CGNS_docs_current/midlevel/solution.html>

pub mod field;

use std::ffi;

use anyhow::{anyhow, Result};
use cgns_sys::*;

use self::field::Field;
use crate::traits::{CGNSNode, CGNSParent};
use crate::utils::{bytes2string, ier_cg_fn, CGNSError, CGIO_NAME_BUFFER_LENGTH};

use super::Zone;

#[derive(Debug, Clone)]
/// CGNS node `FlowSolution_t`
pub struct FlowSolution<'a> {
    pub name: String,
    pub grid_location: GridLocation_t,
    id: i32,
    pub zone: &'a Zone<'a>,
}

impl<'a> FlowSolution<'a> {
    pub fn new_field<S, A>(
        &self,
        datatype: DataType_t,
        name: S,
        solution_array: &[A],
    ) -> Result<i32>
    where
        S: AsRef<str>,
    {
        let mut field_id = 0;
        ier_cg_fn!(cg_field_write(
            self.zone.base.file.id(),
            self.zone.base.id(),
            self.zone.id(),
            self.id(),
            datatype,
            name.as_ref().as_ptr().cast(),
            solution_array.as_ptr().cast(),
            &mut field_id
        ))?;

        Ok(field_id)
    }
}

impl<'a> CGNSNode<'a> for FlowSolution<'a> {
    type Parent = Zone<'a>;

    fn from_id(parent: &'a Self::Parent, id: i32) -> Result<Self> {
        let mut sol_name = [0u8; CGIO_NAME_BUFFER_LENGTH];
        let mut grid_location = GridLocation_t::GridLocationNull;

        ier_cg_fn!(cg_sol_info(
            parent.base.file.id(),
            parent.base.id(),
            parent.id(),
            id,
            sol_name.as_mut_ptr().cast(),
            &mut grid_location
        ))?;
        let name = bytes2string(&sol_name)?;

        Ok(FlowSolution {
            name,
            grid_location,
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

impl<'a> CGNSParent<'a, Field<'a>> for FlowSolution<'a> {
    fn num_child(&self) -> Result<i32> {
        let mut number = 0;
        ier_cg_fn!(cg_nfields(
            self.zone.base.file.id(),
            self.zone.base.id(),
            self.zone.id(),
            self.id,
            &mut number
        ))?;
        Ok(number)
    }
}
