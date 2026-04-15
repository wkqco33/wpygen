use crate::models::ProjectSpec;

pub(crate) fn build_proto(spec: &ProjectSpec) -> String {
    format!(
        r#"syntax = "proto3";

package {package_name};

service Greeter {{
  rpc SayHello(HelloRequest) returns (HelloReply);
}}

message HelloRequest {{
  string name = 1;
}}

message HelloReply {{
  string message = 1;
}}
"#,
        package_name = spec.package_name,
    )
}

pub(crate) fn build_codegen_script(spec: &ProjectSpec) -> String {
    format!(
        r#"from __future__ import annotations

from pathlib import Path

from grpc_tools import protoc

ROOT_DIR = Path(__file__).resolve().parents[1]
PROTO_DIR = ROOT_DIR / "proto"
OUT_DIR = ROOT_DIR / "src" / "{package_name}" / "grpc"
PROTO_FILE = PROTO_DIR / "{package_name}.proto"


def main() -> int:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    args = [
        "",
        f"-I{{PROTO_DIR}}",
        f"--python_out={{OUT_DIR}}",
        f"--grpc_python_out={{OUT_DIR}}",
        str(PROTO_FILE),
    ]
    return int(protoc.main(args))


if __name__ == "__main__":
    raise SystemExit(main())
"#,
        package_name = spec.package_name,
    )
}
