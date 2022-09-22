app "hello"
    packages { pf: "examples/hello-world/platform/main.roc" }
    imports []
    provides [main] to pf

main = "I'm a \(val) Roc application!\n"

rec = { foo: 42, bar: 2.46 }

rec2 = { rec & foo: 24 }

rec3 = &foo rec 123

val = Num.toStr rec3.foo