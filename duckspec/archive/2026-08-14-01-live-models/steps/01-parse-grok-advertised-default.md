# Parse grok advertised default

Parse handshake `currentModelId` onto `InitResult` and expose it as Grok's advertised
default when that id is in the advertised list.

## Tasks

- [x] 1. Parse `modelState.currentModelId` onto `InitResult` in the shared ACP client

- [x] 2. Cache that id on `GrokProvider` and return it from `advertised_default`

- [x] 3. @spec harness/grok Model discovery: Advertised default is the handshake current model when that id is listed
