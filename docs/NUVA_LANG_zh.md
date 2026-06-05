# Nuva 编程语言参考文档

> 版本 1.0 — 2026年5月

## 1. 概述

Nuva 是一门为 Nuva OS 上构建以 UI 为中心的应用而设计的声明式编程语言。其核心理念包括：

- **声明式范式** — 描述"是什么"，而非"怎么做"
- **响应式数据流** — 通过 `signal`/`effect` 实现自动变更传播
- **声明式并发** — `async`/`await` 编译为状态机
- **声明式资源管理** — `resource`/`with` 实现 RAII 风格的自动清理
- **Pipeline 与推导式语法** — 函数式数据变换
- **类型安全** — 静态类型检查与类型推断

Nuva 源代码文件使用 `.nv` 扩展名。

---

## 2. 设计哲学

### 2.1 声明式优先于命令式

Nuva 用声明式等价物替代命令式模式：

| 命令式模式 | Nuva 声明式等价物 |
|-----------|-----------------|
| `new Widget()` + `build()` | `component` 声明 |
| `setState()` + `notify()` | `signal` 自动传播 |
| `addEventListener()` | 修饰符绑定 `.on_click()` |
| `Thread.start()` + 回调 | `async`/`await` |
| `try { acquire() } finally { release() }` | `resource`/`with` |
| `.then().then()` Promise 链 | `await` 链 |

### 2.2 零成本抽象

声明式构造编译为高效运行时表示：

- `component` → 调和后的元素树（O(n) 差分）
- `signal` → 原子版本计数器 + 脏标记
- `effect` → 依赖图 + 拓扑调度
- `async` → 状态机 IR（简单情况无堆分配）
- `resource`/`with` → 作用域化获取/释放，保证清理

### 2.3 安全性

- 静态类型检查与推断
- effect 函数体的纯度验证
- 编译时声明式约束验证
- 资源清理保证（无泄漏）

---

## 3. 关键字参考

### 3.1 核心关键字

| 关键字 | 类别 | 说明 |
|--------|------|------|
| `fn` | 声明 | 函数声明 |
| `let` | 绑定 | 不可变绑定 |
| `var` | 绑定 | 可变绑定 |
| `const` | 绑定 | 编译时常量 |
| `type` | 声明 | 类型别名 |
| `struct` | 声明 | 结构体类型 |
| `enum` | 声明 | 枚举类型 |
| `trait` | 声明 | 特质（接口） |
| `impl` | 声明 | 实现 |
| `if` | 控制 | 条件分支 |
| `else` | 控制 | 备选分支 |
| `match` | 控制 | 模式匹配 |
| `loop` | 控制 | 无限循环 |
| `while` | 控制 | 条件循环 |
| `for` | 控制 | 迭代循环 |
| `break` | 控制 | 退出循环 |
| `continue` | 控制 | 跳过当前迭代 |
| `return` | 控制 | 函数返回 |
| `true` | 字面量 | 布尔真 |
| `false` | 字面量 | 布尔假 |
| `self` | 绑定 | 自引用 |
| `super` | 绑定 | 父作用域 |
| `use` | 模块 | 导入 |
| `mod` | 模块 | 模块声明 |
| `pub` | 可见性 | 公开可见 |
| `as` | 转换 | 类型转换/别名 |

### 3.2 声明式关键字

| 关键字 | 类别 | 说明 |
|--------|------|------|
| `component` | 声明式 | 声明 UI 组件 |
| `signal` | 声明式 | 声明响应式状态变量 |
| `effect` | 声明式 | 注册响应式副作用 |
| `reactive` | 声明式 | 标记函数为响应式安全 |
| `async` | 并发 | 标记函数为异步 |
| `await` | 并发 | 挂起直到 Future 解析完成 |
| `resource` | 资源 | 声明带获取/释放的资源类型 |
| `with` | 资源 | 作用域化资源绑定，自动清理 |

### 3.3 Pipeline 关键字

| 关键字 | 类别 | 说明 |
|--------|------|------|
| `pipeline` | Pipeline | 声明数据处理管道 |
| `yield` | Pipeline | 在管道/生成器中发射值 |
| `filter` | 推导式 | 推导式中的过滤子句 |
| `map` | 推导式 | 推导式中的变换子句 |

---

## 4. 类型系统

### 4.1 基本类型

| 类型 | 说明 | 大小 |
|------|------|------|
| `Int` | 有符号整数 | 平台相关（32/64位） |
| `Int8` | 有符号8位整数 | 1 字节 |
| `Int16` | 有符号16位整数 | 2 字节 |
| `Int32` | 有符号32位整数 | 4 字节 |
| `Int64` | 有符号64位整数 | 8 字节 |
| `UInt` | 无符号整数 | 平台相关 |
| `UInt8` | 无符号8位整数 | 1 字节 |
| `UInt16` | 无符号16位整数 | 2 字节 |
| `UInt32` | 无符号32位整数 | 4 字节 |
| `UInt64` | 无符号64位整数 | 8 字节 |
| `Float` | 浮点数 | 8 字节 (f64) |
| `Float32` | 单精度浮点 | 4 字节 |
| `Bool` | 布尔值 | 1 字节 |
| `Char` | Unicode 字符 | 4 字节 |
| `String` | UTF-8 字符串 | 可变长 |
| `Unit` | 单元类型 `()` | 0 字节 |

### 4.2 特殊类型

| 类型 | 说明 |
|------|------|
| `Reactive<T>` | 响应式包装器 — 读取触发依赖追踪，写入传播到 effect |
| `Future<T>` | 异步计算结果 — 通过 `await` 解析 |
| `Resource<T>` | 受管理资源 — 保证获取/释放生命周期 |
| `Result<T, E>` | 成功（`Ok(T)`）或错误（`Err(E)`） |
| `Option<T>` | 存在（`Some(T)`）或不存在（`None`） |

### 4.3 集合类型

| 类型 | 说明 |
|------|------|
| `Vec<T>` | 动态数组 |
| `HashMap<K, V>` | 哈希映射（SipHash，链地址法，自动 rehash） |
| `HashSet<T>` | 哈希集合 |
| `LinkedList<T>` | 双向链表 |
| `String` | UTF-8 字符串（可增长） |

### 4.4 类型推断

Nuva 支持类型推断，在需要时可显式标注：

```nuva
let x = 42              // 推断为 Int
let y: Float = 3.14     // 显式 Float 标注
let z = x + y           // 推断为 Float（隐式转换）

signal count: Int = 0   // signal 需要显式类型
```

---

## 5. 声明式 UI 范式

### 5.1 组件声明

```nuva
component Greeting(name: String) {
    Column {
        Text("你好, " + name)
            .font_size(24)
            .font_weight(Bold)
            .padding(16)
    }
}
```

### 5.2 组件组合

```nuva
component App() {
    Column {
        Header(title: "我的应用")
        Content()
        Footer()
    }
}
```

### 5.3 条件与循环渲染

```nuva
component TodoList(items: Vec<Todo>) {
    Column {
        if items.is_empty() {
            Text("暂无项目")
        } else {
            for item in items {
                TodoRow(todo: item)
            }
        }
    }
}
```

### 5.4 修饰符链

修饰符可链式调用，应用于布局、样式、事件和无障碍：

```nuva
Text("提交")
    .font_size(16)
    .font_color(Color.Blue)
    .padding(8, 16)
    .background(Color.White)
    .border_radius(4)
    .on_click(handle_submit)
```

---

## 6. 响应式绑定

### 6.1 Signal 声明

```nuva
signal username: String = ""
signal is_logged_in: Bool = false
```

### 6.2 Signal 变更

```nuva
fn on_login(name: String) {
    username = name
    is_logged_in = true
}
```

### 6.3 Effect 注册

```nuva
effect {
    // 初始运行一次，之后每当 username 变更时重新运行
    console.log("用户: " + username)
}

effect {
    if is_logged_in {
        fetch_profile(username)
    }
}
```

### 6.4 响应式计算

```nuva
signal first_name: String = "Alice"
signal last_name: String = "Smith"

effect {
    // 由依赖项响应式计算
    let full_name = first_name + " " + last_name
    update_display(full_name)
}
```

### 6.5 响应式规则

1. **Signal 读取被追踪** — 在 `effect` 体内读取 `signal` 创建依赖
2. **写入传播** — 修改 `signal` 触发所有依赖的 effect
3. **Effect 被调度** — 多个 effect 被批量执行，按拓扑排序
4. **无无限循环** — 调度器检测并阻止循环依赖

---

## 7. 声明式并发

### 7.1 异步函数

```nuva
async fn load_user(id: Int) -> Result<User, Error> {
    let response = await http.get("/api/users/" + id.to_string())
    let user = await response.json()
    return user
}
```

### 7.2 并发组合

```nuva
async fn load_dashboard() -> Dashboard {
    let user = load_user(current_user_id)
    let posts = load_posts()
    let stats = load_stats()

    // 三个请求并发运行
    return Dashboard(
        user: await user,
        posts: await posts,
        stats: await stats
    )
}
```

### 7.3 异步错误处理

```nuva
async fn safe_fetch(url: String) -> Result<Data, Error> {
    match await http.get(url) {
        Ok(response) if response.status == 200 => Ok(await response.json()),
        Ok(response) => Err(Error.HttpError(response.status)),
        Err(e) => Err(e),
    }
}
```

### 7.4 编译为状态机

Nuva 编译器将 `async` 函数变换为状态机 IR：

1. 每个 `await` 点成为一个挂起点
2. 局部变量被捕获到状态机结构体中
3. 恢复执行从下一个状态继续
4. 简单的单 `await` 函数无需堆分配

---

## 8. 声明式资源管理

### 8.1 资源声明

```nuva
resource DatabaseConnection(config: DbConfig) {
    acquire: db.connect(config),
    release: conn.close()
}
```

### 8.2 使用 `with` 的作用域化用法

```nuva
with (conn = DatabaseConnection(default_config)) {
    let result = conn.query("SELECT * FROM users")
    process(result)
}
// conn.close() 在此处自动调用
```

### 8.3 嵌套资源

```nuva
with (conn = DatabaseConnection(config)) {
    with (tx = conn.begin_transaction()) {
        tx.execute("INSERT INTO users ...")
        tx.execute("UPDATE counters ...")
        tx.commit()
    }
    // 若未调用 commit，tx 将回滚
}
// conn 已关闭
```

### 8.4 资源保证

1. **获取** — 进入 `with` 作用域时执行 `acquire` 表达式
2. **释放** — 退出 `with` 作用域时执行 `release` 表达式，即使发生异常
3. **无泄漏** — 资源不能移出其 `with` 作用域
4. **顺序** — 嵌套资源按逆序释放（LIFO）

---

## 9. Pipeline 与推导式语法

### 9.1 管道操作符

```nuva
let result = data
    |> filter(|x| x > 0)
    |> map(|x| x * 2)
    |> reduce(0, |acc, x| acc + x)
```

### 9.2 推导式语法

```nuva
let squares = [x * x for x in 0..10 if x % 2 == 0]
// [0, 4, 16, 36, 64]
```

### 9.3 Pipeline 声明

```nuva
pipeline EtlPipeline {
    stage Extract {
        yield read_csv("input.csv")
    }

    stage Transform {
        yield row
            |> normalize()
            |> validate()
    }

    stage Load {
        write_db(row)
    }
}
```

### 9.4 Pipeline IR

编译器为 Pipeline 生成专用 IR：

- 每个 `stage` 成为独立的 IR 节点
- `yield` 创建挂起/恢复点
- 数据在阶段间流式传输，无需中间集合
- 推导式在可能时优化以避免分配

---

## 10. 运算符参考

### 10.1 算术运算符

| 运算符 | 名称 | 优先级 | 结合性 |
|--------|------|--------|--------|
| `^` | 幂运算 | 7（最高） | 右结合 |
| `*` | 乘法 | 6 | 左结合 |
| `/` | 除法 | 6 | 左结合 |
| `%` | 取模 | 6 | 左结合 |
| `+` | 加法 | 5 | 左结合 |
| `-` | 减法 | 5 | 左结合 |

### 10.2 比较运算符

| 运算符 | 名称 | 优先级 |
|--------|------|--------|
| `==` | 等于 | 4 |
| `!=` | 不等于 | 4 |
| `<` | 小于 | 4 |
| `>` | 大于 | 4 |
| `<=` | 小于等于 | 4 |
| `>=` | 大于等于 | 4 |

### 10.3 逻辑运算符

| 运算符 | 名称 | 优先级 | 结合性 |
|--------|------|--------|--------|
| `not` | 逻辑非 | 3（前缀） | — |
| `and` | 逻辑与 | 2 | 左结合 |
| `or` | 逻辑或 | 1（最低） | 左结合 |

### 10.4 管道运算符

| 运算符 | 名称 | 优先级 |
|--------|------|--------|
| `\|>` | 管道 | 0（最低） |

---

## 11. 文件格式

### 11.1 文件扩展名

所有 Nuva 源代码文件使用 `.nv` 扩展名：

```
my_app/
├── main.nv
├── components/
│   ├── header.nv
│   ├── footer.nv
│   └── sidebar.nv
├── services/
│   └── api.nv
└── styles/
    └── theme.nv
```

### 11.2 模块系统

```nuva
// 从其他模块导入
use components.header.Header
use services.api.{fetch_user, fetch_posts}

// 重导出
pub use components.*
```

### 11.3 入口点

Nuva 应用的入口点是包含 `main` 函数的 `main.nv`：

```nuva
fn main() {
    App().render()
}
```

---

## 12. 编译管线

### 12.1 阶段

```
.nv 源代码
    │
    ▼
┌─────────┐
│  词法分析 │  分词：多进制数字、声明式关键字
└─────────┘
    │
    ▼
┌─────────┐
│  语法分析 │  Pratt 优先级解析，声明式语法（component/signal/effect/async/resource/with）
└─────────┘
    │
    ▼
┌──────────────┐
│  语义分析     │  类型检查、类型推断、纯度验证、声明式约束
└──────────────┘
    │
    ▼
┌──────────────┐
│  代码生成     │  Pipeline IR、推导式 IR、async 状态机 IR、响应式 IR
└──────────────┘
    │
    ▼
┌──────────────┐
│  IR 优化器    │  常量折叠、DCE、CSE、拷贝传播、循环优化、内联
└──────────────┘
    │
    ▼
┌──────────────┐
│  后端         │  VM 字节码或原生代码生成（NEX 格式）
└──────────────┘
```

### 12.2 运行时

| 组件 | 说明 |
|------|------|
| VM | 256 寄存器虚拟机，指令分发 |
| GC | 标记-清除垃圾回收，根扫描 |
| 响应式调度器 | 依赖图、拓扑排序、effect 批处理 |
| NEX 加载器 | 二进制模块加载与重定位 |

---

## 13. 标准库

### 13.1 核心

- `Vec<T>`, `String`, `HashMap<K, V>`, `LinkedList<T>`
- `Option<T>`, `Result<T, E>`
- `Int`, `Float`, `Bool`, `Char`

### 13.2 IO

- `Stdin`, `Stdout`, `Stderr`
- `File`（read, write, open, close）
- `Path`

### 13.3 数学

- 三角函数：`sin`, `cos`, `tan`
- 指数/对数：`exp`, `log`, `log2`, `log10`
- 幂运算：`pow`, `sqrt`
- 舍入：`floor`, `ceil`, `round`

### 13.4 响应式

- `signal`, `effect`, `reactive`
- `Reactive<T>` 类型

### 13.5 异步

- `Future<T>`, `spawn`, `await`
- `Channel<T>` 异步消息传递

### 13.6 资源

- `Resource<T>`, `with`
- 内建资源：`FileHandle`, `DatabaseConnection`, `NetworkSocket`

### 13.7 集合

- `Vec<T>`：`push`, `pop`, `iter`, `map`, `filter`, `reduce`
- `HashMap<K, V>`：`insert`, `get`, `remove`, `contains`, `iter`
- `String`：`concat`, `split`, `trim`, `contains`, `to_upper`, `to_lower`

---

**最后更新**：2026年5月30日
