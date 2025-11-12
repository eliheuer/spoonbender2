// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Spoonbender: A font editor built with Xilem

use xilem::EventLoop;
use xilem::winit::error::EventLoopError;

fn main() -> Result<(), EventLoopError> {
    spoonbender::run(EventLoop::with_user_event())
}
