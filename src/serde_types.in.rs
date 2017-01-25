// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

#[cfg(test)]
mod test_types {
    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub enum T {
        A(usize),
        B,
        C(i8, i8),
        D { a: isize, b: String },
    }
}
