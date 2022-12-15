use rayon::prelude::*;

/// 迭代器
pub fn r_iter() {
    let res = vec![2,8,9]
        .par_iter()
        .map(|n| n*n)
        .sum::<i32>();
    println!("res is {}",res);

    // 声明可以为任何Vec
    let mut left = vec![];
    let mut right = vec![-1];
    (10..15).into_par_iter()
        .enumerate()
        .unzip_into_vecs(&mut left, &mut right);
    // left表示下标  right表示下标值
    println!("{:?}",left);
    println!("{:?}",right);
}


/// rayon scope 可以使用上下文变量
pub fn r_scope() {
    let mut  r1 = "hello world";
    rayon::scope(|s| {
        println!("we see r1 {}",r1);
        s.spawn(|s1|{
            r1 = "hello world s1";
            s1.spawn(|_| {
                println!("we see r1 in s1 inner {}",r1);
            });
        });
    });
}

mod sort {
    use rand::distributions::Standard;
    use rand::Rng;

    pub trait Joiner {
        fn is_paraller() -> bool;
        fn join<A,RA,B,RB>(a:A,b:B) ->(RA,RB)
        where
            A: FnOnce() -> RA + Send,
            B: FnOnce() -> RB + Send,
            RA: Send,
            RB: Send;
    }

    pub struct Paraller;
    
    impl Joiner for Paraller {
        #[inline]
        fn is_paraller() -> bool {
            true
        }

        #[inline]
        fn join<A,RA,B,RB>(a:A,b:B) ->(RA,RB)
                where
                    A: FnOnce() -> RA + Send,
                    B: FnOnce() -> RB + Send,
                    RA: Send,
                    RB: Send,
        {
            rayon::join(a, b)
        }
    }


    pub fn quick_sort<J:Joiner, T: PartialOrd + Send>(v:&mut [T]) {
        if  v.len() <= 1 {
            return;
        }
        let mid = partition(v);
        let (lo,hi)  = v.split_at_mut(mid);
        J::join(|| quick_sort::<J,T>(lo),|| quick_sort::<J,T>(hi));
    }

    fn partition<T: PartialOrd + Send>(v: &mut [T]) -> usize {
        let pivot = v.len() - 1;
        let mut i = 0;
        for j in 0..pivot {
            if v[j] <= v[pivot] {
                v.swap(i, j);
                i += 1;
            }
        }
        v.swap(i, pivot);
        i
    }

    fn seeded_rng() -> rand_xorshift::XorShiftRng {
        use rand::SeedableRng;
        use rand_xorshift::XorShiftRng;
        let mut seed = <XorShiftRng as SeedableRng>::Seed::default();
        (0..).zip(seed.as_mut()).for_each(|(i, x)| *x = i);
        XorShiftRng::from_seed(seed)
    }

    fn default_vec(n:usize) -> Vec<u32> {
        let rng = seeded_rng();
        rng.sample_iter(&Standard).take(n).collect()
    }

    fn is_sorted<T:PartialOrd + Send>(v: &[T]) -> bool {
        (1..v.len()).all(|i| v[i - 1] <= v[i])
    }

    #[cfg(test)]
    mod tests {
        use crate::sort::*;

        #[test]
        fn test_sort() {
            use super::quick_sort;
            // 250_000_000 / 512
            let mut v = default_vec(250_000_000 / 512);
            quick_sort::<Paraller,u32>(&mut v);
            assert_eq!(is_sorted(&v),true);
        }
    }
}


