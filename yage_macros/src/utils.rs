use std::{fmt, mem::MaybeUninit};

pub struct ArrayPushError<T>(pub T);

impl<T> fmt::Debug for ArrayPushError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ArrayPushError").finish_non_exhaustive()
    }
}

pub struct ArrayNotFull(());

pub struct Array<T, const N: usize> {
    inner: [MaybeUninit<T>; N],
    idx: usize,
}

impl<T, const N: usize> Array<T, N> {
    pub const fn new() -> Self {
        Self {
            inner: [const { MaybeUninit::uninit() }; N],
            idx: 0,
        }
    }

    pub fn push(&mut self, elem: T) -> Result<(), ArrayPushError<T>> {
        let idx = self.idx;
        if idx >= N {
            return Err(ArrayPushError(elem));
        }

        let slot = &mut self.inner[idx];
        unsafe {
            slot.as_mut_ptr().write(elem);
        }
        self.idx += 1;
        Ok(())
    }

    pub unsafe fn assume_init_unchecked(self) -> [T; N] {
        if self.idx != N {
            dbg!(self.idx);
            unsafe { std::hint::unreachable_unchecked() }
        }
        unsafe { (&raw const self.inner).cast::<[T; N]>().read() }
    }

    pub fn assume_init(self) -> Result<[T; N], ArrayNotFull> {
        if self.idx != N - 1 {
            return Err(ArrayNotFull(()));
        }
        unsafe { Ok(self.assume_init_unchecked()) }
    }

    pub fn push_rest_with(&mut self, elem: T)
    where
        T: Copy,
    {
        while let Ok(_) = self.push(elem) {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array() {
        let mut array = Array::<u8, 6>::new();
        array.push(1).unwrap();
        array.push(0).unwrap();
        array.push(1).unwrap();
        array.push_rest_with(3);
        assert!(array.push(1).is_err());
        let array = unsafe { array.assume_init_unchecked() };
        assert!(array == [1, 0, 1, 3, 3, 3]);
    }
}
