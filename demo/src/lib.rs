// Copyright 2022 Quentin Gliech
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::borrow_deref_ref,
    clippy::used_underscore_binding,
    clippy::needless_pass_by_value
)]

use std::convert::Infallible;

use bytes::Bytes;
use http::{Request, Response};
use pyo3::prelude::*;
use serde::Deserialize;

use pyo3_matrix_synapse_module::{parse_config, ModuleApi};

#[pyclass]
#[derive(Deserialize)]
struct Config {
    path: String,
}

#[pyclass]
pub struct DemoModule;

#[pymethods]
impl DemoModule {
    #[new]
    fn new(config: &Config, module_api: ModuleApi) -> PyResult<Self> {
        let service = tower::service_fn(|_request: Request<Bytes>| async move {
            let response = Response::new(Bytes::from_static(b"Hello, Rust!"));
            Ok::<_, Infallible>(response)
        });

        module_api.register_web_service(&config.path, service)?;
        Ok(Self)
    }

    #[staticmethod]
    fn parse_config(config: &PyAny) -> PyResult<Config> {
        parse_config(config)
    }
}

#[pymodule]
fn demo(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DemoModule>()?;
    Ok(())
}
