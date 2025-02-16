# super_pool
A super pool that maintains the pool of objects that can be used. Tracks the life times of the objects.

# Pool Library
[`lib.rs`](src/lib.rs) and [`pool_lib.rs`](src/pool_lib.rs) are building blocks for this. Those implementations have their own flaws and they led me to arrive at the implementation in [`pool_lib2.rs`](src/lib2/pool_lib2.rs).

This module provides a generic object pooling mechanism using Rust's type system and concurrency primitives. It allows for the reuse of objects to reduce the overhead of frequent allocations and deallocations.

## Overview

The main components of this module are:

1. `AnyPool` trait: A trait for dynamic downcasting.
2. `ObjectPool<T>` struct: A generic object pool that stores objects of type `T`.
3. `PooledObject<T>` struct: A wrapper around pooled objects that ensures they are returned to the pool when dropped.
4. `SuperPool` struct: A container for multiple `ObjectPool` instances, identified by a string ID.

## Components

### AnyPool Trait

The `AnyPool` trait is used for dynamic downcasting. It allows the `SuperPool` to store different types of `ObjectPool` instances in a single collection.

```rust
trait AnyPool {
    fn as_any(&self) -> &dyn std::any::Any;
}
```

### ObjectPool<T> Struct
The ObjectPool<T> struct is a generic pool that stores objects of type T. It uses an Arc<Mutex<Vec<Arc<T>>>> to manage the pool of objects.

```rust
pub struct ObjectPool<T>
where
    T: Default,
{
    pool: Arc<Mutex<Vec<Arc<T>>>>,
}

impl<T> ObjectPool<T>
where
    T: Default,
{
    pub fn new(data: Vec<Arc<T>>) -> Self {
        ObjectPool {
            pool: Arc::new(Mutex::new(data)),
        }
    }

    fn get(&self) -> PooledObject<T> {
        let data = self.pool.lock().unwrap().pop();
        PooledObject {
            pool: Arc::clone(&self.pool),
            value: data.unwrap_or_else(|| {
                let tmp = Arc::new(T::default());
                self.pool.lock().unwrap().push(tmp.clone());
                tmp
            }),
        }
    }
}
```
### PooledObject<T> Struct
The PooledObject<T> struct wraps a pooled object and ensures it is returned to the pool when dropped.

```rust
struct PooledObject<T> {
    value: Arc<T>,
    pool: Arc<Mutex<Vec<Arc<T>>>>,
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        let mut pool = self.pool.lock().unwrap();
        pool.push(Arc::clone(&self.value));
    }
}
```
### SuperPool Struct
The SuperPool struct manages multiple ObjectPool instances, identified by a string ID. It uses a HashMap to store the pools.

```rust
struct SuperPool {
    pool_map: HashMap<String, Arc<Box<dyn AnyPool>>>,
}

impl SuperPool {
    pub fn new() -> Self {
        SuperPool {
            pool_map: HashMap::new(),
        }
    }

    pub fn add_pool<T: 'static>(&mut self, pool_id: &str, object_pool: ObjectPool<T>)
    where
        T: Default,
    {
        self.pool_map
            .insert(pool_id.to_string(), Arc::new(Box::new(object_pool)));
    }

    pub fn get_data<T: 'static>(&mut self, pool_id: &str) -> Option<PooledObject<T>>
    where
        T: Default,
    {
        if let Some(pool) = self.pool_map.get(pool_id) {
            let tmp = pool.as_any().downcast_ref::<ObjectPool<T>>();
            if let Some(pool) = tmp {
                return Some(pool.get());
            }
        }
        None
    }
}
```

### Helper function to print the pool
A note about the helper function here. If the parameter is passed as an object and not a reference, the temporaries are destroyed as soon
as they are passed to the function and the objects gets pushed back to the pool.

```rust
fn print_me<T: Debug + Any>(v: &Option<PooledObject<T>>) {
    match v {
        Some(p) => println!("Got {:?}", p.value),
        None => println!("Got nothing..."),
    }
}
```

### Conclusion
This module provides a flexible and efficient way to manage object pools in Rust. By using generic types and dynamic downcasting, it allows for the reuse of objects of different types, reducing the overhead of frequent allocations and deallocations.