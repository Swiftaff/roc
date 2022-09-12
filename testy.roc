app "hello"
    packages { pf: "examples/interactive/cli-platform/main.roc" }
    imports [pf.Stdout]
    provides [main] to pf

main = Stdout.line "I'm a \(test) application!"

rec = { foo: 42, bar: 2.46 }

rec2 = { rec & foo: 24 }

fooUpdater : { foo : I8, bar : F32 }, I8 -> { foo : I8, bar : F32 }
fooUpdater = \r, v -> { r & foo : v }

# rec3 = &foo rec2 12

rec3 = fooUpdater rec2 12

test = Num.toStr rec3.foo