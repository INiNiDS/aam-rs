<?php

declare(strict_types=1);

namespace RustGames\Aam;

use RuntimeException;
use FFI;
use FFI\CData;

/**
 * High-performance PHP FFI wrapper for the AAM Rust parser.
 */
final class AamDocument
{
    private FFI $ffi;
    private ?CData $handle = null;

    public function __construct(string $content, ?string $libPath = null)
    {
        if (!class_exists('FFI')) {
            throw new RuntimeException('PHP FFI extension is not enabled. Вруби ffi.enable=true в php.ini!');
        }

        $lib = $libPath ?? getenv('AAM_RS_LIB') ?: __DIR__ . '/../../target/release/libaam_rs.so';

        $this->ffi = FFI::cdef(<<<'CDEF'
            typedef struct AamHandle AamHandle;
            typedef struct AamInlineObjectHandle AamInlineObjectHandle;
            AamHandle* aam_new(void);
            void aam_free(AamHandle* handle);
            int aam_parse(AamHandle* handle, const char* content);
            int aam_load(AamHandle* handle, const char* path);
            int aam_update(AamHandle* handle);
            int aam_reload(AamHandle* handle, const char* content);
            char* aam_format(AamHandle* handle, const char* content);

            char* aam_get(AamHandle* handle, const char* key);
            char* aam_find(AamHandle* handle, const char* query);
            char* aam_deep_search(AamHandle* handle, const char* pattern);
            char* aam_reverse_search(AamHandle* handle, const char* value);

            char* aam_schema_names(AamHandle* handle);
            char* aam_type_names(AamHandle* handle);

            int aam_reconstruct_push(AamHandle* handle, const char* content);
            char* aam_reconstruct_schema(const AamHandle* handle, const char* schema_name);
            void aam_reconstruct_clear(AamHandle* handle);

            void aam_string_free(char* s);
            const char* aam_last_error(AamHandle* handle);

            AamInlineObjectHandle* aam_inline_new(void);
            void aam_inline_free(AamInlineObjectHandle* handle);
            int aam_inline_add(AamInlineObjectHandle* handle, const char* key, const char* value);
            char* aam_inline_to_string(const AamInlineObjectHandle* handle);
            char* aam_parse_inline_to_map(const char* content);
        CDEF, $lib);

        $this->handle = $this->ffi->aam_new();
        if (FFI::isNull($this->handle)) {
            throw new RuntimeException('Failed to allocate AAM handle');
        }

        $rc = $this->ffi->aam_parse($this->handle, $content);
        if ($rc !== 0) {
            $this->throwLastError('Native parse failed');
        }
    }

    public static function parse(string $content, ?string $libPath = null): self
    {
        return new self($content, $libPath);
    }

    /**
     * @return array<string, AamBuilder>
     */
    public static function splitAam(string $content): array
    {
        $result = [];
        $currentName = null;
        $currentBuilder = null;

        foreach (preg_split('/\r?\n/', $content) ?: [] as $rawLine) {
            $line = trim($rawLine);
            if ($line === '') {
                continue;
            }

            $header = self::parseSectionHeader($line);
            if ($header !== null) {
                if ($currentName !== null && $currentBuilder !== null) {
                    $result[$currentName] = $currentBuilder;
                }
                $currentName = $header;
                $currentBuilder = new AamBuilder();
                continue;
            }

            if ($currentName === null || $currentBuilder === null) {
                continue;
            }

            $assignment = self::parseAssignment($line);
            if ($assignment !== null) {
                [$key, $value] = $assignment;
                $currentBuilder->addLine($key, $value);
            }
        }

        if ($currentName !== null && $currentBuilder !== null) {
            $result[$currentName] = $currentBuilder;
        }

        return $result;
    }

    public function __destruct()
    {
        if ($this->handle !== null && !FFI::isNull($this->handle)) {
            $this->ffi->aam_free($this->handle);
            $this->handle = null;
        }
    }

    public function reload(string $content): void
    {
        $this->ffi->aam_reload($this->handle, $content);
    }

    /**
     * Reload the document from its original on-disk source file (the path
     * captured at load time). Throws if this instance was not loaded from a
     * file path.
     */
    public function update(): void
    {
        $rc = $this->ffi->aam_update($this->handle);
        if ($rc !== 0) {
            $this->throwLastError('Native update failed');
        }
    }

    public function format(string $content): string
    {
        $formattedPtr = $this->ffi->aam_format($this->handle, $content);
        if ($formattedPtr === null) {
            $this->throwLastError('Native format failed');
        }

        try {
            return FFI::string($formattedPtr);
        } finally {
            $this->ffi->aam_string_free($formattedPtr);
        }
    }

    public function get(string $key): ?string
    {
        $valuePtr = $this->ffi->aam_get($this->handle, $key);
        if ($valuePtr === null) {
            return null;
        }

        try {
            return FFI::string($valuePtr);
        } finally {
            $this->ffi->aam_string_free($valuePtr);
        }
    }

    public function reverseSearch(string $value): array
    {
        $ptr = $this->ffi->aam_reverse_search($this->handle, $value);
        return $this->parseCList($ptr);
    }

    public function find(string $query): array
    {
        $ptr = $this->ffi->aam_find($this->handle, $query);
        return $this->parseCMap($ptr);
    }

    public function deepSearch(string $pattern): array
    {
        $ptr = $this->ffi->aam_deep_search($this->handle, $pattern);
        return $this->parseCMap($ptr);
    }

    public function schemaNames(): array
    {
        $ptr = $this->ffi->aam_schema_names($this->handle);
        return $this->parseCList($ptr);
    }

    public function typeNames(): array
    {
        $ptr = $this->ffi->aam_type_names($this->handle);
        return $this->parseCList($ptr);
    }

    /**
     * Reconstruct a @schema directive from a list of AAM content strings.
     * @return string Formatted @schema definition.
     */
    public function reconstructSchema(string $name, array $contents): string
    {
        $this->ffi->aam_reconstruct_clear($this->handle);
        foreach ($contents as $content) {
            $rc = $this->ffi->aam_reconstruct_push($this->handle, $content);
            if ($rc !== 0) {
                $this->throwLastError('Native push failed');
            }
        }

        $ptr = $this->ffi->aam_reconstruct_schema($this->handle, $name);
        if ($ptr === null) {
            $this->throwLastError('Native schema reconstruction failed');
        }

        try {
            return FFI::string($ptr);
        } finally {
            $this->ffi->aam_string_free($ptr);
        }
    }

    private function throwLastError(string $defaultMsg): void
    {
        $err = $this->ffi->aam_last_error($this->handle);
        $msg = $err !== null ? FFI::string($err) : $defaultMsg;
        throw new RuntimeException($msg);
    }

    private function parseCList(?CData $ptr): array
    {
        if ($ptr === null) {
            return [];
        }
        try {
            $str = FFI::string($ptr);
            return $str === '' ? [] : explode(',', $str);
        } finally {
            $this->ffi->aam_string_free($ptr);
        }
    }

    private function parseCMap(?CData $ptr): array
    {
        if ($ptr === null) {
            return [];
        }
        try {
            $str = FFI::string($ptr);
            if ($str === '') {
                return [];
            }
            $result = [];
            foreach (explode("\n", $str) as $line) {
                $parts = explode('=', $line, 2);
                if (count($parts) === 2) {
                    $result[$parts[0]] = $parts[1];
                }
            }
            return $result;
        } finally {
            $this->ffi->aam_string_free($ptr);
        }
    }

    private static function parseSectionHeader(string $line): ?string
    {
        if (!str_starts_with($line, '#')) {
            return null;
        }

        $rest = trim(substr($line, 1));
        return str_ends_with($rest, '.aam') ? $rest : null;
    }

    /**
     * @return array{0: string, 1: string}|null
     */
    private static function parseAssignment(string $line): ?array
    {
        $idx = strpos($line, '=');
        if ($idx === false || $idx <= 0) {
            return null;
        }

        $key = trim(substr($line, 0, $idx));
        if ($key === '') {
            return null;
        }

        return [$key, trim(substr($line, $idx + 1))];
    }
}

final class SchemaField
{
    private function __construct(
        public readonly string $name,
        public readonly string $typeName,
        public readonly bool $optional,
    ) {
    }

    public static function required(string $name, string $typeName): self
    {
        return new self($name, $typeName, false);
    }

    public static function optional(string $name, string $typeName): self
    {
        return new self($name, $typeName, true);
    }

    public function toAam(): string
    {
        return $this->optional ? "{$this->name}*: {$this->typeName}" : "{$this->name}: {$this->typeName}";
    }
}

final class AamBuilder
{
    /** @var list<string> */
    private array $lines = [];

    public function addLine(string $key, string $value): self
    {
        $this->lines[] = "$key = $value";
        return $this;
    }

    public function comment(string $text): self
    {
        $this->lines[] = "# $text";
        return $this;
    }

    /** @param list<SchemaField> $fields */
    public function schema(string $name, array $fields): self
    {
        $rendered = array_map(static fn (SchemaField $field): string => $field->toAam(), $fields);
        $this->lines[] = '@schema ' . $name . ' { ' . implode(', ', $rendered) . ' }';
        return $this;
    }

    /** @param list<string> $schemas */
    public function derive(string $path, array $schemas = []): self
    {
        $suffix = $schemas === [] ? '' : '::' . implode('::', $schemas);
        $this->lines[] = '@derive ' . $path . $suffix;
        return $this;
    }

    public function import(string $path): self
    {
        $this->lines[] = '@import ' . $path;
        return $this;
    }

    public function typeAlias(string $alias, string $typeName): self
    {
        $this->lines[] = '@type ' . $alias . ' = ' . $typeName;
        return $this;
    }

    public function build(): string
    {
        return implode("\n", $this->lines);
    }

    public function toFile(string $path): void
    {
        file_put_contents($path, $this->build());
    }

    /** @param InlineObject $obj */
    public function addInline(string $key, InlineObject $obj): self
    {
        $this->addLine($key, (string)$obj);
        return $this;
    }
}

final class InlineObject
{
    /** @var list<string> */
    private array $pairs = [];

    public function add(string $key, string $value): self
    {
        $this->pairs[] = "$key = $value";
        return $this;
    }

    public function __toString(): string
    {
        return '{ ' . implode(', ', $this->pairs) . ' }';
    }
}

/**
 * Parse an inline object string into a PHP associative array.
 *
 * @return array<string, string>
 */
function parse_inline_to_map(string $content): array
{
    $ffi = FFI::cdef(<<<'CDEF'
        char* aam_parse_inline_to_map(const char* content);
        void aam_string_free(char* s);
    CDEF, getenv('AAM_RS_LIB') ?: __DIR__ . '/../../target/release/libaam_rs.so');

    $cContent = FFI::new('char[' . (strlen($content) + 1) . ']', false);
    FFI::memcpy($cContent, $content, strlen($content));
    $cContent[strlen($content)] = "\0";

    $ptr = $ffi->aam_parse_inline_to_map(FFI::cast('char*', $cContent));
    if ($ptr === null) {
        throw new RuntimeException('Failed to parse inline object');
    }

    try {
        $str = FFI::string($ptr);
        if ($str === '') {
            return [];
        }
        $result = [];
        foreach (explode("\n", $str) as $line) {
            $parts = explode('=', $line, 2);
            if (count($parts) === 2) {
                $result[$parts[0]] = $parts[1];
            }
        }
        return $result;
    } finally {
        $ffi->aam_string_free($ptr);
    }
}
