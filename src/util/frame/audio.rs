use std::mem;
use std::slice;
use std::ops::{Deref, DerefMut};

use libc::{c_int, int64_t, c_ulonglong};
use ffi::*;
use ::ChannelLayout;
use ::util::format;
use super::Frame;

#[derive(PartialEq, Eq)]
pub struct Audio(Frame);

impl Audio {
	pub unsafe fn wrap(ptr: *mut AVFrame) -> Self {
		Audio(Frame::wrap(ptr))
	}

	pub unsafe fn alloc(&mut self, format: format::Sample, samples: usize, layout: ChannelLayout) {
		self.set_format(format);
		self.set_samples(samples);
		self.set_channel_layout(layout);

		av_frame_get_buffer(self.as_mut_ptr(), 1);
	}
}

impl Audio {
	pub fn empty() -> Self {
		unsafe {
			Audio(Frame::empty())
		}
	}

	pub fn new(format: format::Sample, samples: usize, layout: ChannelLayout) -> Self {
		unsafe {
			let mut frame = Audio::empty();
			frame.alloc(format, samples, layout);

			frame
		}
	}

	pub fn format(&self) -> format::Sample {
		unsafe {
			if (*self.as_ptr()).format == -1 {
				format::Sample::None
			}
			else {
				format::Sample::from(mem::transmute::<_, AVSampleFormat>(((*self.as_ptr()).format)))
			}
		}
	}

	pub fn set_format(&mut self, value: format::Sample) {
		unsafe {
			(*self.as_mut_ptr()).format = mem::transmute::<AVSampleFormat, c_int>(value.into());
		}
	}

	pub fn channel_layout(&self) -> ChannelLayout {
		unsafe {
			ChannelLayout::from_bits_truncate(av_frame_get_channel_layout(self.as_ptr()) as c_ulonglong)
		}
	}

	pub fn set_channel_layout(&mut self, value: ChannelLayout) {
		unsafe {
			av_frame_set_channel_layout(self.as_mut_ptr(), value.bits() as int64_t);
		}
	}

	pub fn channels(&self) -> u16 {
		unsafe {
			av_frame_get_channels(self.as_ptr()) as u16
		}
	}

	pub fn set_channels(&mut self, value: u16) {
		unsafe {
			av_frame_set_channels(self.as_mut_ptr(), value as c_int);
		}
	}

	pub fn rate(&self) -> u32 {
		unsafe {
			av_frame_get_sample_rate(self.as_ptr()) as u32
		}
	}

	pub fn set_rate(&mut self, value: u32) {
		unsafe {
			av_frame_set_sample_rate(self.as_mut_ptr(), value as c_int);
		}
	}

	pub fn samples(&self) -> usize {
		unsafe {
			(*self.as_ptr()).nb_samples as usize
		}
	}

	pub fn set_samples(&mut self, value: usize) {
		unsafe {
			(*self.as_mut_ptr()).nb_samples = value as c_int;
		}
	}

	pub fn is_planar(&self) -> bool {
		self.format().is_planar()
	}

	pub fn is_packed(&self) -> bool {
		self.format().is_packed()
	}

	pub fn planes(&self) -> usize {
		unsafe {
			if (*self.as_ptr()).linesize[0] == 0 {
				return 0;
			}
		}

		if self.is_packed() {
			1
		}
		else {
			self.channels() as usize
		}
	}

	pub fn plane<T: Sample>(&self, index: usize) -> &[T] {
		if index >= self.planes() {
			panic!("out of bounds");
		}

		if !<T as Sample>::is_valid(self.format()) {
			panic!("unsupported type");
		}

		unsafe {
			slice::from_raw_parts(
				mem::transmute((*self.as_ptr()).data[index]),
				mem::size_of::<T>() * self.samples())
		}
	}

	pub fn plane_mut<T: Sample>(&mut self, index: usize) -> &[T] {
		if index >= self.planes() {
			panic!("out of bounds");
		}

		if !<T as Sample>::is_valid(self.format()) {
			panic!("unsupported type");
		}

		unsafe {
			slice::from_raw_parts_mut(
				mem::transmute((*self.as_mut_ptr()).data[index]),
				mem::size_of::<T>() * self.samples())
		}
	}

	pub fn data(&self) -> Vec<&[u8]> {
		let mut result = Vec::new();

		unsafe {
			for i in 0 .. self.planes() {
				result.push(slice::from_raw_parts(
					(*self.as_ptr()).data[i],
					(*self.as_ptr()).linesize[0] as usize));
			}
		}

		result
	}

	pub fn data_mut(&mut self) -> Vec<&mut [u8]> {
		let mut result = Vec::new();

		unsafe {
			for i in 0 .. self.planes() {
				result.push(slice::from_raw_parts_mut(
					(*self.as_mut_ptr()).data[i],
					(*self.as_ptr()).linesize[0] as usize));
			}
		}

		result
	}
}

impl Deref for Audio {
	type Target = Frame;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

impl DerefMut for Audio {
	fn deref_mut(&mut self) -> &mut<Self as Deref>::Target {
		&mut self.0
	}
}

impl Clone for Audio {
	fn clone(&self) -> Self {
		let mut cloned = Audio::new(self.format(), self.samples(), self.channel_layout());
		cloned.clone_from(self);

		cloned
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			av_frame_copy(self.as_mut_ptr(), source.as_ptr());
			av_frame_copy_props(self.as_mut_ptr(), source.as_ptr());
		}
	}
}

pub trait Sample {
	fn is_valid(format: format::Sample) -> bool;
}

impl Sample for u8 {
	fn is_valid(format: format::Sample) -> bool {
		if let format::Sample::U8(..) = format {
			true
		}
		else {
			false
		}
	}
}

impl Sample for i16 {
	fn is_valid(format: format::Sample) -> bool {
		if let format::Sample::I16(..) = format {
			true
		}
		else {
			false
		}
	}
}

impl Sample for i32 {
	fn is_valid(format: format::Sample) -> bool {
		if let format::Sample::I32(..) = format {
			true
		}
		else {
			false
		}
	}
}

impl Sample for f32 {
	fn is_valid(format: format::Sample) -> bool {
		if let format::Sample::F32(..) = format {
			true
		}
		else {
			false
		}
	}
}

impl Sample for f64 {
	fn is_valid(format: format::Sample) -> bool {
		if let format::Sample::F64(..) = format {
			true
		}
		else {
			false
		}
	}
}
