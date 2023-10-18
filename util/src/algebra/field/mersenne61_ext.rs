use super::Field;
use core::arch::x86_64::_mulx_u64;
use rand::Rng;

/// GitHub Copilot: `Mersenne61Ext`结构体是一个表示扩展Mersenne素数的有限域上的元素的类型。
/// 它是一个64位有符号整数的扩展类型，它的模数是一个Mersenne素数，即$2^{61}-1$。
/// 这个类型实现了一些基本的算术运算，如加法、减法、乘法和逆元运算，以及一些其他的功能，如指数运算和随机元素生成。
#[derive(Debug, Clone, Copy)]
pub struct Mersenne61Ext {
    real: u64,
    image: u64,
}

const MOD: u64 = (1u64 << 61) - 1;

impl Mersenne61Ext {
    pub fn get_real(&self) -> u64 {
        self.real
    }
}

#[inline]
fn try_sub(x: u64) -> u64 {
    if x >= MOD {
        x - MOD
    } else {
        x
    }
}

impl std::ops::Neg for Mersenne61Ext {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            real: try_sub(self.real ^ MOD),
            image: try_sub(self.image ^ MOD),
        }
    }
}

impl std::ops::Add for Mersenne61Ext {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            real: try_sub(self.real + rhs.real),
            image: try_sub(self.image + rhs.image),
        }
    }
}

impl std::ops::AddAssign for Mersenne61Ext {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub for Mersenne61Ext {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut real = self.real + (rhs.real ^ MOD);
        if real >= MOD {
            real -= MOD;
        }
        let mut image = self.image + (rhs.image ^ MOD);
        if image >= MOD {
            image -= MOD;
        }
        Mersenne61Ext { real, image }
    }
}

impl std::ops::SubAssign for Mersenne61Ext {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

#[inline]
fn my_mult(x: u64, y: u64) -> u64 {
    let mut hi = 0;
    let lo = unsafe { _mulx_u64(x, y, &mut hi) };
    ((hi << 3) | (lo >> 61)) + (lo & MOD)
}

#[inline]
fn my_mod(x: u64) -> u64 {
    (x >> 61) + (x & MOD)
}

impl std::ops::Mul for Mersenne61Ext {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let all_prod = my_mult(self.real + self.image, rhs.real + rhs.image);
        let ac = my_mult(self.real, rhs.real);
        let bd = MOD * 2 - my_mult(self.image, rhs.image);
        let nac = MOD * 2 - ac;

        let mut t_img = my_mod(all_prod + nac + bd);
        if t_img >= MOD {
            t_img -= MOD;
        }
        let mut t_real = my_mod(ac + bd);
        t_real = try_sub(t_real);
        Mersenne61Ext {
            real: t_real,
            image: t_img,
        }
    }
}

impl std::ops::MulAssign for Mersenne61Ext {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl std::fmt::Display for Mersenne61Ext {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "real: {}, image: {}", self.real, self.image)
    }
}

impl std::cmp::PartialEq for Mersenne61Ext {
    fn eq(&self, rhs: &Self) -> bool {
        self.real == rhs.real && self.image == rhs.image
    }
}

impl Field for Mersenne61Ext {
    const LOG_ORDER: u64 = 62;
    const ROOT_OF_UNITY: Mersenne61Ext = Mersenne61Ext {
        real: 2147483648,
        image: 1033321771269002680,
    };
    const INVERSE_2: Self = Mersenne61Ext {
        real: 1152921504606846976,
        image: 0,
    };

    #[inline]
    fn from_int(x: u64) -> Self {
        Mersenne61Ext { real: x, image: 0 }
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.real == 0 && self.image == 0
    }

    #[inline]
    fn random_element() -> Self {
        let mut rng = rand::thread_rng();
        Mersenne61Ext {
            real: rng.gen_range(0..MOD),
            image: rng.gen_range(0..MOD),
        }
    }

    fn inverse(&self) -> Self {
        let p = 2305843009213693951u128;
        let mut n = p * p - 2;
        let mut ret = Self::from_int(1);
        let mut base = self.clone();
        while n != 0 {
            if n % 2 == 1 {
                ret *= base;
            }
            base *= base;
            n >>= 1;
        }
        ret
    }

    #[inline]
    fn to_bytes(&self) -> Vec<u8> {
        let x = self.real.to_le_bytes().to_vec();
        x
    }
}

// #[cfg(test)]
// mod tests {
//     use super::super::field_tests::*;
//     use super::*;

//     #[test]
//     fn test() {
//         add_and_sub::<Mersenne61Ext>();
//         mult_and_inverse::<Mersenne61Ext>();
//         assigns::<Mersenne61Ext>();
//         pow_and_generator::<Mersenne61Ext>();
//     }
// }
