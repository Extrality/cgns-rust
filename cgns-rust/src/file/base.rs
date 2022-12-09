pub mod zone;

use cgns_sys::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use self::zone::Zone;

use super::File;
use crate::{
    traits::{CGNSNode, CGNSParent},
    utils::{bytes2string, ier_cg_fn, Result, CGIO_NAME_BUFFER_LENGTH},
};

#[derive(Debug, Clone, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum CellDimension {
    Line = 1,
    Surface = 2,
    Volume = 3,
}

#[derive(Debug, Clone, PartialEq)]
/// CGNS node `CGNSBase_t`
pub struct Base<'a> {
    pub name: String,
    /// Dimension of the cells
    pub cell_dim: CellDimension,
    /// Number of coordinates required to define a vector in the field.
    pub phys_dim: i32,
    pub id: i32,
    pub file: &'a File,
}

impl<'a> Base<'a> {
    // TODO: DimensionalUnits_t (cg_unitsfull_read)
}

impl<'a> CGNSNode<'a> for Base<'a> {
    type Parent = File;

    fn from_id(parent: &'a Self::Parent, id: i32) -> Result<Self> {
        let mut name = [0u8; CGIO_NAME_BUFFER_LENGTH];
        let mut cell_dim = 0;
        let mut phys_dim = 0;

        ier_cg_fn!(cg_base_read(
            parent.id(),
            id,
            name.as_mut_ptr().cast(),
            &mut cell_dim,
            &mut phys_dim
        ))?;
        let name = bytes2string(&name)?;
        let cell_dim = CellDimension::try_from(cell_dim).unwrap();

        Ok(Base {
            name,
            cell_dim,
            phys_dim,
            id,
            file: parent,
        })
    }
    fn id(&self) -> i32 {
        self.id
    }
    fn parent(&self) -> &Self::Parent {
        self.file
    }
}

impl<'a> CGNSParent<'a, Zone<'a>> for Base<'a> {
    fn num_child(&self) -> Result<i32> {
        let mut number = 0;
        ier_cg_fn!(cg_nzones(self.file.id(), self.id, &mut number))?;
        Ok(number)
    }
}
