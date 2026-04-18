# Backend i18n Scheme

后端不直接承担完整多语言渲染，建议只负责返回稳定的业务语义，前端基于 `vue-i18n` 做最终文案展示。

## Recommended Response Shape

成功响应保持现有数据结构；失败响应统一成下面这个格式：

```json
{
  "success": false,
  "error": {
    "code": "device.pending_approval",
    "message": "Device is pending approval",
    "params": {
      "approvalUrl": "https://example.com/devices/123/review"
    }
  }
}
```

字段约定：

- `code`: 稳定错误码，前后端都依赖它，禁止直接拿中文文案当协议。
- `message`: 后端兜底消息。前端找不到 `code` 时可以直接显示它。
- `params`: 插值参数，给前端翻译模板使用。

## Frontend Mapping Rule

前端优先按 `code` 映射翻译 key：

```ts
const message = t(`backend.${error.code}`, error.params) || error.message;
```

建议保留两层兜底：

1. `backend.${error.code}` 存在时，使用前端语言包。
2. key 不存在时，退回后端返回的 `message`。
3. `message` 也没有时，退回通用 `errors.unknown`。

## Error Code Naming

建议统一为小写、点分结构：

- `auth.token_invalid`
- `auth.token_expired`
- `device.pending_approval`
- `device.disabled`
- `activity.validation_failed`
- `inspiration.image_too_large`

不要把 HTTP 状态码直接当业务码使用。HTTP 只表达传输层语义，`code` 表达业务语义。

## Server-side Guideline

后端内部抛错时，尽量在边界层转成统一结构：

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiErrorPayload {
    code: String,
    message: String,
    params: Option<serde_json::Value>,
}
```

返回建议：

- 参数校验失败：HTTP `400`
- 鉴权失败：HTTP `401` / `403`
- 资源不存在：HTTP `404`
- 业务冲突：HTTP `409`
- 待审核这类“可恢复但不能继续”的状态：可继续用 HTTP `202`，并配套 `device.pending_approval`
- 未知异常：HTTP `500`

## Why This Scheme

- 后端不需要维护整套多语言文案。
- 前端可以统一走 `vue-i18n`，语言切换即时生效。
- 错误码稳定后，桌面端、Web 端、移动端都能复用同一套协议。
- 新增语言时只补前端语言包，不需要改后端业务逻辑。
