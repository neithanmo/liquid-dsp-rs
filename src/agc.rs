use libc::c_uint;
use std::fmt;

use num::complex::Complex32;
use std::mem::transmute;

use crate::enums::AgcSquelchMode;
use crate::liquid_dsp_sys as raw;

use crate::utils::{LiquidComplex, LiquidFloatComplex};

pub struct AgcCrcf {
    inner: raw::agc_crcf,
    is_locked: bool,
}

pub struct AgcRrrf {
    inner: raw::agc_rrrf,
    is_locked: bool,
}

macro_rules! agc_xxx_impl {
    ($obj:ty, ($create:expr, $reset:expr,
        $lock:expr, $unlock:expr,
        $setband:expr, $getband:expr,
        $setsignal:expr, $getsignal:expr, 
        $setrssi:expr, $getrssi:expr,
        $setgain:expr, $getgain:expr,
        $setscale:expr, $getscale:expr,
        $squelche:expr, $squelchd:expr,
        $squelch:expr,$setthres:expr,
        $getthres:expr, $settimeout:expr,
        $gettimeout:expr, $status:expr,
        $destroy:expr)) => {
        impl $obj {
            pub fn create() -> Self {
                Self {
                    inner: unsafe { $create() },
                    is_locked: false,
                }
            }

            pub fn reset(&mut self) {
                unsafe {
                    $reset(self.inner);
                }
            }

            pub fn lock(&mut self) {
                unsafe {
                    $lock(self.inner);
                    self.is_locked = true;
                }
            }

            pub fn unlock(&mut self) {
                unsafe {
                    $unlock(self.inner);
                    self.is_locked = false;
                }
            }

            pub fn set_bandwidth(&mut self, b: f32) -> Result<(), &'static str> {
                if b < 0f32 {
                    return Err("bandwith must be positive");
                }
                unsafe {
                    $setband(self.inner, b);
                }
                Ok(())
            }

            pub fn get_bandwidth(&self) -> f32 {
                unsafe { $getband(self.inner) }
            }

            pub fn get_signal_level(&self) -> f32 {
                unsafe { $getsignal(self.inner) }
            }

            pub fn set_signal_level(&mut self, level: f32) -> Result<(), &'static str> {
                if level <= 0f32 {
                    return Err("level must be greater than zero");
                }
                unsafe {
                    $setsignal(self.inner, level);
                }
                Ok(())
            }
            pub fn get_rssi(&self) -> f32 {
                unsafe { $getrssi(self.inner) }
            }

            pub fn set_rssi(&mut self, rssi: f32) {
                unsafe {
                    $setrssi(self.inner, rssi);
                }
            }

            pub fn get_gain(&self) -> f32 {
                unsafe { $getgain(self.inner) }
            }

            pub fn set_gain(&mut self, gain: f32) -> Result<(), &'static str> {
                if gain <= 0f32 {
                    return Err("gain must be greater than zero");
                }
                unsafe {
                    $setgain(self.inner, gain);
                }
                Ok(())
            }

            pub fn get_scale(&self) -> f32 {
                unsafe { $getscale(self.inner) }
            }

            pub fn set_scale(&mut self, scale: f32) -> Result<(), &'static str> {
                if scale <= 0f32 {
                    return Err("scale must be greater than zero");
                }
                unsafe {
                    $setscale(self.inner, scale);
                }
                Ok(())
            }

            pub fn squelch_enable(&mut self) {
                unsafe {
                    $squelche(self.inner);
                }
            }

            pub fn squelch_disable(&mut self) {
                unsafe {
                    $squelchd(self.inner);
                }
            }

            pub fn squelch_is_enabled(&self) -> bool {
                unsafe { $squelch(self.inner) == 1 }
            }

            pub fn squelch_set_threshold(&self, th: f32) {
                unsafe {
                    $setthres(self.inner, th);
                }
            }

            pub fn squelch_get_threshold(&self) -> f32 {
                unsafe { $getthres(self.inner) }
            }

            pub fn squelch_set_timeout(&self, timeout: u64) {
                unsafe {
                    $settimeout(self.inner, timeout as c_uint);
                }
            }

            pub fn squelch_get_timeout(&self) -> u64 {
                unsafe { $gettimeout(self.inner) as u64 }
            }

            pub fn squelch_status(&self) -> AgcSquelchMode {
                unsafe { AgcSquelchMode::from_bits($status(self.inner) as u8).unwrap() }
            }
        }

        impl fmt::Debug for $obj {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let bandwith = self.get_bandwidth();
                let locked = if self.is_locked { "yes" } else { "no" };
                let status = match self.squelch_status() {
                    AgcSquelchMode::DISABLED => "disabled",
                    _ => "enabled",
                };
                let rssi = self.get_rssi();
                let scale = self.get_scale();
                let gain = if scale > 0f32 {
                    10.0 * scale.log10()
                } else {
                    -100.0
                };
                write!(
                    f,
                    "agc [rssi: {} dB, output gain: {} dB, bw: {}, locked: {}, squelch: {}]:\n",
                    rssi, gain, bandwith, locked, status
                )
            }
        }

        impl Drop for $obj {
            fn drop(&mut self) {
                unsafe {
                    $destroy(self.inner);
                }
            }
        }
    };
}

agc_xxx_impl!(
    AgcCrcf,
    (
        raw::agc_crcf_create,
        raw::agc_crcf_reset,
        raw::agc_crcf_lock,
        raw::agc_crcf_unlock,
        raw::agc_crcf_set_bandwidth,
        raw::agc_crcf_get_bandwidth,
        raw::agc_crcf_set_signal_level,
        raw::agc_crcf_get_signal_level,
        raw::agc_crcf_set_rssi,
        raw::agc_crcf_get_rssi,
        raw::agc_crcf_set_gain,
        raw::agc_crcf_get_gain,
        raw::agc_crcf_set_scale,
        raw::agc_crcf_get_scale,
        raw::agc_crcf_squelch_enable,
        raw::agc_crcf_squelch_disable,
        raw::agc_crcf_squelch_is_enabled,
        raw::agc_crcf_squelch_set_threshold,
        raw::agc_crcf_squelch_get_threshold,
        raw::agc_crcf_squelch_set_timeout,
        raw::agc_crcf_squelch_get_timeout,
        raw::agc_crcf_squelch_get_status,
        raw::agc_crcf_destroy
    )
);

agc_xxx_impl!(
    AgcRrrf,
    (
        raw::agc_rrrf_create,
        raw::agc_rrrf_reset,
        raw::agc_rrrf_lock,
        raw::agc_rrrf_unlock,
        raw::agc_rrrf_set_bandwidth,
        raw::agc_rrrf_get_bandwidth,
        raw::agc_rrrf_set_signal_level,
        raw::agc_rrrf_get_signal_level,
        raw::agc_rrrf_set_rssi,
        raw::agc_rrrf_get_rssi,
        raw::agc_rrrf_set_gain,
        raw::agc_rrrf_get_gain,
        raw::agc_rrrf_set_scale,
        raw::agc_rrrf_get_scale,
        raw::agc_rrrf_squelch_enable,
        raw::agc_rrrf_squelch_disable,
        raw::agc_rrrf_squelch_is_enabled,
        raw::agc_rrrf_squelch_set_threshold,
        raw::agc_rrrf_squelch_get_threshold,
        raw::agc_rrrf_squelch_set_timeout,
        raw::agc_rrrf_squelch_get_timeout,
        raw::agc_rrrf_squelch_get_status,
        raw::agc_rrrf_destroy
    )
);

impl AgcCrcf {
    pub fn init(&mut self, input: &mut [Complex32]) -> Result<(), &'static str> {
        if input.len() == 0 {
            return Err("number of samples must be greater than zero");
        }
        unsafe {
            raw::agc_crcf_init(
                self.inner,
                transmute::<*mut Complex32, *mut LiquidFloatComplex>(input.as_mut_ptr()),
                input.len() as c_uint,
            );
        }
        Ok(())
    }

    pub fn execute(&self, mut input: Complex32) -> Complex32 {
        let mut out = Complex32::default();
        let ptr = &mut out as *mut Complex32;
        unsafe {
            // this is safe because Complex<T> reproduce c
            let inp =
            transmute::<*mut Complex32, *mut LiquidFloatComplex>(&mut input as *mut Complex32);
            raw::agc_crcf_execute(
                self.inner,
                *inp,
                transmute::<*mut Complex32, *mut LiquidFloatComplex>(ptr),
            );
            *ptr
        }
    }

    pub fn execute_block(&self, input: &mut [Complex32], output: &mut [Complex32]) {
        assert!(
            input.len() == output.len(),
            "Input and output buffers with different length"
        );
        unsafe {
            raw::agc_crcf_execute_block(
                self.inner,
                transmute::<*mut Complex32, *mut LiquidFloatComplex>(input.as_mut_ptr()),
                input.len() as c_uint,
                transmute::<*mut Complex32, *mut LiquidFloatComplex>(output.as_mut_ptr()),
            );
        }
    }
}

impl AgcRrrf {
    pub fn init(&mut self, input: &mut [f32]) -> Result<(), &'static str> {
        if input.len() == 0 {
            return Err("number of samples must be greater than zero");
        }
        unsafe {
            raw::agc_rrrf_init(self.inner, input.as_mut_ptr(), input.len() as c_uint);
        }
        Ok(())
    }

    pub fn execute(&self, input: f32) -> f32 {
        let ptr = &mut 0f32 as *mut f32;
        unsafe {
            raw::agc_rrrf_execute(self.inner, input, ptr);
            *ptr
        }
    }

    pub fn execute_block(&self, input: &mut [f32], output: &mut [f32]) {
        assert!(
            input.len() == output.len(),
            "Input and output buffers with different length"
        );
        unsafe {
            raw::agc_rrrf_execute_block(
                self.inner,
                input.as_mut_ptr(),
                input.len() as c_uint,
                output.as_mut_ptr(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AgcCrcf, AgcRrrf};
    use num::complex::Complex32;
    use num::Zero;

    #[test]
    fn test_agc_crcf_execute_block() {
        let mut input = Vec::with_capacity(4);
        let mut output = vec![Complex32::zero(); 4];
        for i in 0..4 {
            input.push(Complex32::new(2.0 + i as f32 * 2.0, -2.8 * 0.5 * i as f32));
        }
        let mut agc = AgcCrcf::create();
        agc.set_bandwidth(0.001).unwrap();
        agc.set_gain(0.5).unwrap();
        agc.set_scale(1.5).unwrap();
        agc.squelch_enable();
        agc.execute_block(&mut input, &mut output);
        let solution = [
            Complex32::new(1.5, -0.0),
            Complex32::new(3.0, -1.05),
            Complex32::new(4.4999924, -2.0999963),
            Complex32::new(5.9999495, -3.1499734),
        ];
        assert_eq!(&output, &solution);
    }

    #[test]
    fn test_agc_crcf_rssi() {
        let agc = AgcCrcf::create();
        agc.execute(Complex32::new(5.9999495, -3.1499734));
        let rssi = agc.get_rssi();
        assert_eq!(0.016113421, rssi);    
    }

   /* #[test]
    fn test_agc_crcf_init() {
        let mut input = Vec::with_capacity(4);
        let mut output = vec![Complex32::zero(); 4];
        let mut s = 0f32;
        for i in 0..4 {
            let val = Complex32::new(2.0 + i as f32 * 2.0, -2.8 * 0.5 * i as f32);
            input.push(val);
        }

        let mut agc = AgcCrcf::create();
        agc.init(&mut input);
        agc.set_bandwidth(0.001).unwrap();
        agc.set_gain(0.5).unwrap();
        agc.set_scale(1.5).unwrap();
        agc.squelch_enable(); 
        println!("{:?}", agc);
        assert_eq!(1, 1);  
    }*/

}