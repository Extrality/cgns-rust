use cgns_rust::traits::CGNSParent;
use cgns_rust::{self};

// Used for testing during alpha versions development
fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    let f = cgns_rust::file::File::new(&args[1], cgns_rust::file::OpenFileMode::Read).unwrap();
    for b in f.iter().unwrap() {
        println!("{}", b.name);
        for z in b.iter().unwrap() {
            println!("  {}", z.name);
            for elem in z.iter_elements().unwrap() {
                println!("    E {} ({:?})", elem.name, elem.elem_type);
                if format!("{:?}", elem.elem_type) == "NFACE_n"
                    || format!("{:?}", elem.elem_type) == "NGON_n"
                {
                    let txt = format!("{:?}", elem.read().unwrap());
                    println!("{}", &txt[0..200]);
                }
            }
            // for grid in z.iter_grid_coordinates().unwrap() {
            //     println!("    G {}", grid.name);
            //     for coords in grid.iter().unwrap() {
            //         println!("        {}", coords.name);
            //     }
            // }
            // for flow_solution in z.iter_solutions().unwrap() {
            //     println!("    S {}", flow_solution.name);
            //     for field in flow_solution.iter().unwrap() {
            //         let data = field.read().unwrap();
            //         let print_len = data.len().min(5);
            //         println!(
            //             "        {}({}):  {:?}",
            //             field.name,
            //             data.len(),
            //             &data[0..print_len]
            //         );
            //     }
            // }
        }
    }
}
