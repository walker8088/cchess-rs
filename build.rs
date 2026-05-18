// build.rs - 构建脚本
// 目前为空，可用于未来构建时的配置

fn main() {
    // 可以在这里添加构建时的自定义逻辑
    // 例如：生成代码、检查环境变量等

    // 告诉 Cargo 如果这些文件发生变化，需要重新运行构建脚本
    println!("cargo:rerun-if-changed=build.rs");
}
