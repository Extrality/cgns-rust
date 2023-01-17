//! Based on: <https://cgns.github.io/CGNS_docs_current/midlevel/structural.html#base>

pub mod zone;

use std::ffi::CString;

use cgns_sys::*;

use self::zone::Zone;
use super::File;
use crate::traits::{CGNSNode, CGNSNodeIterator, CGNSParent};
use crate::utils::{bytes2string, ier_cg_fn, string2bytes, Result, CGIO_NAME_BUFFER_LENGTH};

#[derive(Debug, Clone, PartialEq)]
/// CGNS node [`CGNSBase_t`](https://cgns.github.io/CGNS_docs_current/sids/cgnsbase.html)
///
/// The master CGNS node. Most `File`s have a single `Base` (fittingly named `Base`).
pub struct Base<'a> {
    pub name: String,
    /// Dimension of the cells
    pub cell_dim: i32,
    /// Number of coordinates required to define a vector in the field.
    pub phys_dim: i32,
    id: i32,
    file: &'a File<'a>,
}

impl<'a> Base<'a> {
    // TODO: DimensionalUnits_t (cg_unitsfull_read)
    pub fn iter_zones(&'a self) -> Result<CGNSNodeIterator<'a, Zone<'a>>> {
        self.iter()
    }

    /// Warning:
    ///
    /// For CGNS files opened in with [`crate::file::OpenFileMode::Modify`],
    /// if a base with the same name already exists, it will be overwritten !
    pub fn write(file: &'a File, name: String, cell_dim: i32, phys_dim: i32) -> Result<Self> {
        let c_name: CString = string2bytes(&name)?;
        let mut id = 0;
        ier_cg_fn!(cg_base_write(
            file.id(),
            c_name.as_ptr(),
            cell_dim,
            phys_dim,
            &mut id
        ))?;
        Ok(Base {
            name,
            cell_dim,
            phys_dim,
            id,
            file,
        })
    }
}

impl<'a> CGNSNode<'a> for Base<'a> {
    type Parent = File<'a>;

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

#[cfg(test)]
mod tests {
    use testdir::testdir;

    use super::super::tests::*;
    use super::*;
    use crate::file::OpenFileMode;
    use crate::library::LibraryHandle;

    #[test]
    fn can_write_base() {
        let name_hw = "HelloWorld".to_string();
        let name_zz = "zozo".to_string();
        let library = LibraryHandle::acquire();

        // 1. Can I write bases ?
        let (p, f) = cgns_file(&library, testdir!(), 0);
        let base_1 = Base::write(&f, name_hw.clone(), 3, 3).unwrap();
        assert_eq!(base_1.id, 1);
        let base_2 = Base::write(&f, name_zz, 1, 2).unwrap();
        assert_eq!(base_2.id, 2);

        // 2. Can I read them back ?
        unsafe { f.close_by_ref().unwrap() };
        let f_bis = library.open(p, OpenFileMode::Modify).unwrap();
        let names: Vec<_> = f_bis.iter().unwrap().map(|n| n.name).collect();
        assert_eq!(
            names.as_slice(),
            ["HelloWorld".to_string(), "zozo".to_string()]
        );
        // 3. Can I overwrite them ?
        let base_1_bis = Base::write(&f_bis, name_hw, 1, 1).unwrap();
        assert_eq!(base_1_bis.id, base_1.id);
        assert_ne!(base_1_bis.cell_dim, base_1.cell_dim);

        // Cleanup
        std::mem::forget(f);
        f_bis.close().unwrap();
    }
}
