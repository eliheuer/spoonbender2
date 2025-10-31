// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Spoonbender: A font editor built with Xilem

use winit::error::EventLoopError;
use xilem::EventLoop;

fn main() -> Result<(), EventLoopError> {
    spoonbender::run(EventLoop::with_user_event())
}
