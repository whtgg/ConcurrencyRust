use std::{cell::{Cell, RefCell, RefMut, OnceCell, LazyCell}, rc::Rc, collections::HashMap};


/// Box 智能指针
/// box 保证不会分配超过isize::MAX的空间
pub fn s_box() {
    #[derive(Debug)]
    enum List<T> {
        Cons(T,Box<List<T>>),
        Nil,
    }
    let list:List<i32> = List::Cons(5, Box::new(List::Cons(3, Box::new(List::Nil))));
    println!("{list:?}");
}

/// 内部可变性
/// 相比Refcell 值必须实现Copy
/// 非线程安全  没有实现Sync
/// 
pub fn s_cell() {
    let shared_map: Rc<RefCell<_>> = Rc::new(RefCell::new(HashMap::new()));
    // {
    //     let mut map: RefMut<_> = shared_map.borrow_mut();
    //     map.insert("africa", 92388);
    //     map.insert("kyoto", 11837);
    //     map.insert("piccadilly", 11826);
    //     map.insert("marbles", 38);
    // }

    let mut map: RefMut<_> = shared_map.borrow_mut();
    map.insert("africa", 92388);
    map.insert("kyoto", 11837);
    map.insert("piccadilly", 11826);
    map.insert("marbles", 38);
    drop(map);

    let total: i32 = shared_map.borrow().values().sum();
    println!("{total}");
}

pub fn s_once_cell() {
    let cell = OnceCell::new();
    assert!(cell.get().is_none());

    let value: &String = cell.get_or_init(|| "Hello, World!".to_string());
    assert_eq!(value, "Hello, World!");
    assert!(cell.get().is_some());
}

pub fn s_lazy_cell() {
    // let cell = LazyCell::new(|| "hello".to_uppercase());
    // assert_eq!(&*cell, "HELLO");

    let lazy: LazyCell<i32> = LazyCell::new(|| {
        println!("initializing");
        92
    });
    println!("ready");
    println!("{}", *lazy);
    println!("{}", *lazy);
    println!("{}", *lazy);
}

/// 使用自定义Rc 
pub fn s_rc() {
    let s = rc::Rc::new("hello world");
    let s1 = s.clone();

    let v = s1.value();
    println!("value: {}", v);
}



pub mod rc {
    use std::cell::Cell;
    use std::marker::PhantomData;
    use std::process::abort;
    use std::ptr::NonNull;

    pub struct Rc<T: ?Sized> {
        ptr: NonNull<RcBox<T>>,
        phantom: PhantomData<RcBox<T>>,
    }

    impl<T> Rc<T> {
        pub fn new(t: T) -> Self {
            let ptr = Box::new(RcBox {
                strong: Cell::new(1),
                value: t,
            });
            let ptr = NonNull::new(Box::into_raw(ptr)).unwrap();
            Self {
                ptr: ptr,
                phantom: PhantomData,
            }
        }

        pub fn value(&self) -> &T {
            &self.inner().value
        }
    }


    struct RcBox<T: ?Sized> {
        strong: Cell<usize>,
        value: T,
    }

    impl<T: ?Sized> Clone for Rc<T> {
        fn clone(&self) -> Rc<T> {
            self.inc_strong();
            Rc {
                ptr: self.ptr,
                phantom: PhantomData,
            }
        }
    }

    trait RcBoxPtr<T: ?Sized> {
        fn inner(&self) -> &RcBox<T>;

        fn strong(&self) -> usize {
            self.inner().strong.get()
        }

        fn inc_strong(&self) {
            self.inner()
                .strong
                .set(self.strong().checked_add(1).unwrap_or_else(|| abort()));
        }
    }

    impl<T: ?Sized> RcBoxPtr<T> for Rc<T> {
        fn inner(&self) -> &RcBox<T> {
            unsafe { self.ptr.as_ref() }
        }
    }
}


