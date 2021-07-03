use std::ops::AddAssign;

#[derive(Default, Debug)]
pub struct PassedTime(i32);

// for `passed_time += 5`
impl AddAssign<i32> for PassedTime{
	fn add_assign(&mut self, rhs:i32 ) {
		self.0 += rhs
	}
}