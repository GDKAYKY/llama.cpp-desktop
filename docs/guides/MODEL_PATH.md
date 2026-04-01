Fair. Here it is **clean English Markdown**, no fluff.

---

## Model Manifest Resolution Flow

### Input

* `{modelPath}` defined by the frontend
* Stored in `userConfig.json`

---

## Manifest Mounting Rules

If the `manifests` folder exists, mount using the following structure:

```
{modelProvider}/{manifestLibrary}/{modelName}/{modelVersion}
```

📌 **ATTENTION:**
`modelVersion` is a **manifest file**, not a directory.

---

## Manifest File (`modelVersion`)

Example manifest content:

```json
{
  "schemaVersion": 2,
  "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
  "config": {
    "mediaType": "application/vnd.docker.container.image.v1+json",
    "digest": "sha256:d9bb33f2786931fea42f50936a2424818aa2f14500638af2f01861eb2c8fb446",
    "size": 487
  },
  "layers": [
    {
      "mediaType": "application/vnd.ollama.image.model",
      "digest": "sha256:60e05f2100071479f596b964f89f510f057ce397ea22f2833a0cfe029bfc2463",
      "size": 4683074048
    },
    {
      "mediaType": "application/vnd.ollama.image.system",
      "digest": "sha256:66b9ea09bd5b7099cbb4fc820f31b575c0366fa439b08245566692c6784e281e",
      "size": 68
    },
    {
      "mediaType": "application/vnd.ollama.image.template",
      "digest": "sha256:1e65450c30670713aa47fe23e8b9662bdf4065e81cc8e3cbfaa98924fcc0d320",
      "size": 1615
    },
    {
      "mediaType": "application/vnd.ollama.image.license",
      "digest": "sha256:832dd9e00a68dd83b3c3fb9f5588dad7dcf337a0db50f7d9483f310cd292e92e",
      "size": 11343
    }
  ]
}
```

---

## Extraction Logic

* Read the manifest file (`modelVersion`)
* Extract required fields
* Persist them into `modelLibrary.json`

---

## Blob Resolution

The `digest` value of the layer with:

```
mediaType = application/vnd.ollama.image.model
```

is resolved to a physical file at:

```
{blobs}/sha256-60e05f2100071479f596b964f89f510f057ce397ea22f2833a0cfe029bfc2463
```

📌 This file is the actual **modelFile**.

---

## Frontend Contract

1. The user selects the directory where models are stored
2. The frontend builds a reference string in the format:

```
{modelProvider}:{modelName}:{modelVersion}
```

### Example

```
ollama:qwen2.5-coder:7b
```

---

## Manifest Path Example

Corresponding manifest path:

```
\manifests\registry.ollama.ai\library\qwen2.5-coder\7b
```

---