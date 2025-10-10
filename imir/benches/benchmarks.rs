// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
//
// SPDX-License-Identifier: MIT

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use imir::parse_targets;

fn benchmark_parse_targets(c: &mut Criterion,)
{
    let yaml = r"
targets:
  - owner: octocat
    repository: hello-world
    type: open_source
    badge:
      style: flat
      widget:
        columns: 2
        alignment: center
        border_radius: 8
  - owner: testuser
    repository: test-repo
    type: open_source
  - owner: example
    type: profile
    slug: example-profile
";

    c.bench_function("parse_targets_small", |b| {
        b.iter(|| parse_targets(black_box(yaml,),).expect("parse failed",),)
    },);
}

fn benchmark_target_normalization(c: &mut Criterion,)
{
    let yaml = r"
targets:
  - owner: user1
    repository: repo1
    type: open_source
  - owner: user2
    repository: repo2
    type: open_source
    badge:
      style: for_the_badge
  - owner: user3
    type: profile
";

    c.bench_function("parse_targets_with_badges", |b| {
        b.iter(|| {
            let doc = parse_targets(black_box(yaml,),).expect("parse failed",);
            black_box(doc.targets.len(),)
        },)
    },);
}

fn benchmark_large_config_parse(c: &mut Criterion,)
{
    let mut yaml = String::from("targets:\n",);
    for i in 0..100 {
        yaml.push_str(&format!(
            "  - owner: user{i}\n    repository: repo{i}\n    type: open_source\n"
        ),);
    }

    c.bench_function("parse_100_targets", |b| {
        b.iter(|| parse_targets(black_box(&yaml,),).expect("parse failed",),)
    },);
}

fn benchmark_yaml_parsing(c: &mut Criterion,)
{
    let complex_yaml = r"
targets:
  - owner: org1
    repository: complex-repo
    type: open_source
    slug: custom-slug
    display_name: Complex Repository
    branch: feature/test
    contributors_branch: develop
    badge:
      style: for_the_badge
      widget:
        columns: 3
        alignment: end
        border_radius: 12
";

    c.bench_function("parse_complex_target", |b| {
        b.iter(|| parse_targets(black_box(complex_yaml,),).expect("parse failed",),)
    },);
}

criterion_group!(
    benches,
    benchmark_parse_targets,
    benchmark_target_normalization,
    benchmark_large_config_parse,
    benchmark_yaml_parsing
);
criterion_main!(benches);
