% 计数

## 重复替代

在宏中计数是一项让人吃惊地难搞的活儿。最简单的方式是采用重复替代。

```rust
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}

macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ replace_expr!($tts 1usize))*};
}
# 
# fn main() {
#     assert_eq!(count_tts!(0 1 2), 3);
# }
```

对于小数目来说，这方法不错，但当输入量到达500标记附近时，编译器将被击溃。想想吧，输出的结果将类似：

```ignore
0usize + 1usize + /* ~500 `+ 1usize`s */ + 1usize
```

编译器必须把这一大串解析成一棵AST，那可会是一棵完美失衡的500多级深的二叉树。

## 递归

递归是个老套路。

```rust
macro_rules! count_tts {
    () => {0usize};
    ($_head:tt $($tail:tt)*) => {1usize + count_tts!($($tail)*)};
}
# 
# fn main() {
#     assert_eq!(count_tts!(0 1 2), 3);
# }
```

> **注意**：对于`rustc`1.2来说，很不幸，编译器在处理大数量的类型未知的整型字面值时将会出现性能问题。我们此处显式采用`usize`类型就是为了避免这种不幸。
>
> 如果这样做并不合适(比如说，当类型必须可替换时)，可通过`as`来减轻问题。(比如，`0 as $ty`, `1 as $ty`等)。

这方法管用，但很快就会超出宏递归的次数限制。与重复替换不同的是，可通过增加匹配分支来增加可处理的输入面值。

```rust
macro_rules! count_tts {
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $_k:tt $_l:tt $_m:tt $_n:tt $_o:tt
     $_p:tt $_q:tt $_r:tt $_s:tt $_t:tt
     $($tail:tt)*)
        => {20usize + count_tts!($($tail)*)};
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $($tail:tt)*)
        => {10usize + count_tts!($($tail)*)};
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $($tail:tt)*)
        => {5usize + count_tts!($($tail)*)};
    ($_a:tt
     $($tail:tt)*)
        => {1usize + count_tts!($($tail)*)};
    () => {0usize};
}

fn main() {
    assert_eq!(700, count_tts!(
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        
        // 重复替换手段差不多将在此处崩溃
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,

        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
    ));
}
```

我们列出的这种形式可以处理差不多1,200个标记。

## 切片长度

第三种方法，是帮助编译器构建一个深度较小的AST，以避免栈溢出。可以通过新建数列，并调用其`len`方法来做到。

```rust
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}

macro_rules! count_tts {
    ($($tts:tt)*) => {<[()]>::len(&[$(replace_expr!($tts ())),*])};
}
# 
# fn main() {
#     assert_eq!(count_tts!(0 1 2), 3);
# }
```

经过测试，这种方法可处理高达10,000个标记数，可能还能多上不少。缺点是，就Rust 1.2来说，没法拿它生成常量表达式。即便结果可以被优化成一个简单的常数(在`debug build`里得到的编译结果仅是一次内存载入)，它仍然无法被用在常量位置(如`const`值或定长数组的长度值)。

不过，如果非常量计数够用的话，此方法很大程度上是上选。

## 枚举计数

当你需要计互不相同的标识符的数量时，可以用到此方法。结果还可被用作常量。

```rust
macro_rules! count_idents {
    ($($idents:ident),* $(,)*) => {
        {
            #[allow(dead_code, non_camel_case_types)]
            enum Idents { $($idents,)* __CountIdentsLast }
            const COUNT: u32 = Idents::__CountIdentsLast as u32;
            COUNT
        }
    };
}
# 
# fn main() {
#     const COUNT: u32 = count_idents!(A, B, C);
#     assert_eq!(COUNT, 3);
# }
```

此方法的确有两大缺陷。其一，如上所述，它仅能被用于数有效的标识符(同时还不能是关键词)，而且它不允许那些标识符有重复。其二，此方法不具备卫生性；就是说如果你的末位标识符(在`__CountIdentsLast`位置的标识符)的字面值也是输入之一，宏调用就会失败，因为`enum`中包含重复变量。
