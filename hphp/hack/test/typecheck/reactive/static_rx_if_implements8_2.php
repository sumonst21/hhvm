<?hh // strict
<<file: __EnableUnstableFeatures('coeffects_provisional')>>

interface Rx {}

class A {
  <<__Rx, __OnlyRxIfImpl(Rx::class)>>
  public function f(): int {
    return 1;
  }
}

class B extends A {
  <<__Rx>>
  public function g(): void {
    // should be an error, B does not implement Rx
    parent::f();
  }
}
