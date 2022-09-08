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

use bytes::{Buf, Bytes};
use http::{Request, Response};
use http_body::{Body, Full};
use pyo3::{exceptions::PyValueError, types::PyType, FromPyObject, PyAny, PyErr, PyResult};
use pyo3_twisted_web::Resource;
use serde::Deserialize;
use tower_service::Service;

pub struct ModuleApi<'a> {
    inner: &'a PyAny,
}

impl<'a> ModuleApi<'a> {
    /// Register a [`Service`] to handle a path
    ///
    /// # Errors
    ///
    /// Returns an error if the call to `ModuleApi.register_web_resource` failed
    pub fn register_web_service<S, B, E>(&self, path: &str, service: S) -> PyResult<()>
    where
        S: Service<Request<Bytes>, Response = Response<B>, Error = E> + Clone + Send + 'static,
        S::Future: Send,
        B: Into<Bytes>,
        E: Into<PyErr> + 'static,
    {
        self.inner
            .call_method1("register_web_resource", (path, Resource::new(service)))?;
        Ok(())
    }
}

impl<'a> FromPyObject<'a> for ModuleApi<'a> {
    fn extract(inner: &'a PyAny) -> PyResult<Self> {
        Ok(Self { inner })
    }
}

/// Convert a dict to `T` via `serde_json`, useful for implementing `parse_config`
///
/// # Errors
///
/// Returns an error if it failed to convert the dict
pub fn parse_config<'a, T: Deserialize<'a>>(config: &'a PyAny) -> PyResult<T> {
    let py = config.py();
    let config: &str = py
        .import("json")?
        .call_method1("dumps", (config,))?
        .extract()?;

    serde_json::from_str(config).map_err(|_| PyValueError::new_err("failed to convert config"))
}
