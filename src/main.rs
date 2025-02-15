mod lib;

use lib::ReusableEnum;

use crate::lib::{ObjectPool, PooledObject, SuperPool};
fn main() {
    let obj_pool1 = ObjectPool::new(
        vec![
            ReusableEnum::Text(String::from("this")),
            ReusableEnum::Text(String::from("is")),
            ReusableEnum::Text(String::from("a")),
            ReusableEnum::Text(String::from("pool")),
        ],
        ReusableEnum::Text(String::from("Default")),
    );

    let obj_pool2 = ObjectPool::new(
        vec![
            ReusableEnum::Int(1),
            ReusableEnum::Int(2),
            ReusableEnum::Int(3),
        ],
        ReusableEnum::Int(0),
    );

    let mut super_pool: SuperPool = SuperPool::new_with_pool(String::from("String"), obj_pool1);
    super_pool.add_to_pool(String::from("int"), obj_pool2);

    string_demo(&mut super_pool);

    int_demo(&mut super_pool);
}

fn int_demo(super_pool: &mut SuperPool) {
    let iv1 = super_pool.get("int");
    let iv2 = super_pool.get("int");
    let iv3 = super_pool.get("int");
    print_me("int", &iv1);
    print_me("int", &iv2);
    print_me("int", &iv3);
    // now we exchausted the items in the pool.
    print_me("int", &super_pool.get("int")); // returns 0.
}

fn string_demo(super_pool: &mut SuperPool) {
    {
        let v1 = super_pool.get("String");
        let v2 = super_pool.get("String");
        print_me("String", &v1);
        print_me("String", &v2);
    }
    // now if we get a value from the pool, we should be getting one of "a"/"pool".
    // This is because, at this point v1 and v2 are out of scope and they should be added back to the pool.
    let v3 = super_pool.get("String");
    print_me("String", &v3);
    let v4 = super_pool.get("String");
    print_me("String", &v4);
    let v5 = super_pool.get("String");
    print_me("String", &v5);
    let v6 = super_pool.get("String");
    print_me("String", &v6);
}

fn print_me(pool_type: &str, v2: &Option<PooledObject>) {
    match v2 {
        Some(x) => println!("For pool_type({}), got the value: {:?}", pool_type, *x),
        None => println!("Got None"),
    }
}
