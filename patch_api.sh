#!/bin/bash
sed -i 's/pub mod media;/pub mod media;\npub mod projects;/' backend/src/api/mod.rs
sed -i 's/\.merge(media::router())/.merge(media::router())\n        .merge(projects::router())/' backend/src/api/mod.rs
