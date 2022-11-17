//! Based on: <https://cgns.github.io/CGNS_docs_current/midlevel/bc.html>

use std::ffi;

use anyhow::{anyhow, Result};
use cgns_sys::*;

use crate::traits::CGNSNode;
use crate::utils::{bytes2string, ier_cg_fn, CGNSError, CGIO_NAME_BUFFER_LENGTH};

use super::Zone;

#[derive(Debug, Clone)]
/// CGNS node `BC_t`
pub struct BC<'a> {
    pub name: String,
    pub bc_type: BCType_t,
    pub point_set_type: PointSetType_t,
    pub nb_points: i64,
    pub normal_index: [i32; 3],
    pub normal_list_flag: bool,
    pub normal_data_type: DataType_t,
    pub nb_datasets: i32,
    id: i32,
    pub zone: &'a Zone<'a>,
}

impl<'a> CGNSNode<'a> for BC<'a> {
    type Parent = Zone<'a>;

    fn from_id(parent: &'a Self::Parent, id: i32) -> Result<Self> {
        let mut bc_name = [0u8; CGIO_NAME_BUFFER_LENGTH];
        let mut bc_type = BCType_t::BCTypeNull;
        let mut point_set_type = PointSetType_t::PointSetTypeNull;
        let mut nb_points = 0;
        let mut normal_index = [0; 3];
        let mut normal_list_flag = 0;
        let mut normal_data_type = DataType_t::DataTypeNull;
        let mut nb_datasets = 0;

        ier_cg_fn!(cg_boco_info(
            parent.base.file.id(),
            parent.base.id(),
            parent.id(),
            id,
            bc_name.as_mut_ptr().cast(),
            &mut bc_type,
            &mut point_set_type,
            &mut nb_points,
            normal_index.as_mut_ptr(),
            &mut normal_list_flag,
            &mut normal_data_type,
            &mut nb_datasets
        ))?;

        let name = bytes2string(&bc_name)?;

        Ok(BC {
            name,
            bc_type,
            point_set_type,
            nb_points,
            normal_index,
            normal_list_flag: normal_list_flag == 1,
            normal_data_type,
            nb_datasets,
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
