<?php

declare(strict_types=1);

final class AamPhp
{
    /** @var object */
    private $ffi;

    public function __construct(?string $libPath = null)
    {
        if (!class_exists('FFI')) {
            throw new RuntimeException('PHP FFI extension is not enabled');
        }

        $lib = $libPath ?? getenv('AAM_RS_LIB') ?: __DIR__ . '/../../target/release/libaam_rs.so';

        $this->ffi = \FFI::cdef(<<<'CDEF'
            typedef struct AamlHandle AamlHandle;
            AamlHandle* aam_new(void);
            void aam_free(AamlHandle* handle);
            int aam_parse(AamlHandle* handle, const char* content);
            char* aam_find_obj(AamlHandle* handle, const char* key);
            void aam_string_free(char* s);
            const char* aam_last_error(AamlHandle* handle);
        CDEF, $lib);
    }

    public function parseFindObj(string $content, string $key): ?string
    {
        $handle = $this->ffi->aam_new();
        if ($handle === null) {
            throw new RuntimeException('Failed to allocate AAML handle');
        }

        try {
            $rc = $this->ffi->aam_parse($handle, $content);
            if ($rc !== 0) {
                $err = $this->ffi->aam_last_error($handle);
                $msg = $err !== null ? \FFI::string($err) : 'Native parse failed';
                throw new RuntimeException($msg);
            }

            $valuePtr = $this->ffi->aam_find_obj($handle, $key);
            if ($valuePtr === null) {
                return null;
            }

            try {
                return \FFI::string($valuePtr);
            } finally {
                $this->ffi->aam_string_free($valuePtr);
            }
        } finally {
            $this->ffi->aam_free($handle);
        }
    }
}


