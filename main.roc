app "hello"
    packages { pf: "examples/interactive/cli-platform/main.roc" }
    imports [pf.Stdout]
    provides [main] to pf

main = Stdout.line "I'm a \(recstr) application!"

rec = { foo: 42, bar: 2.46 }
rec = { rec & bar: 2.4 }
recstr = Num.toStr rec.foo