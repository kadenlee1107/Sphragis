// Bat_OS — Blink/WTF Foundation Test
// Verifies that the WTF layer (String, AtomicString, Vector, HashMap)
// compiles and functions correctly on bare-metal ARM64.

extern "C" {
    #include <stdio.h>
    #include <stdlib.h>
}

#include "third_party/blink/renderer/platform/wtf/text/wtf_string.h"
#include "third_party/blink/renderer/platform/wtf/text/atomic_string.h"
#include "third_party/blink/renderer/platform/wtf/vector.h"
#include "third_party/blink/renderer/platform/wtf/hash_map.h"
#include "base/memory/ref_counted.h"
#include "base/memory/raw_ptr.h"
#include "base/check.h"

extern "C" void _start() {
    printf("\n=== Bat_OS Blink/WTF Foundation Test ===\n");
    printf("Chromium infrastructure on bare-metal ARM64\n\n");

    int passed = 0;

    // [1] WTF::String basics
    {
        WTF::String s("Hello, Blink!");
        printf("[1] WTF::String: \"%s\" len=%u\n", s.impl().c_str(), s.length());
        if (s.length() == 13 && s == "Hello, Blink!") {
            printf("    PASS\n"); passed++;
        }
    }

    // [2] WTF::String operations
    {
        WTF::String s("BatBrowser");
        WTF::String lower = s.LowerASCII();
        printf("[2] LowerASCII: \"%s\" -> \"%s\"\n", s.impl().c_str(), lower.impl().c_str());
        if (lower == "batbrowser") {
            printf("    PASS\n"); passed++;
        }
    }

    // [3] WTF::StringBuilder
    {
        WTF::StringBuilder sb;
        sb.Append("Bat");
        sb.Append('_');
        sb.Append("OS v");
        sb.AppendNumber(3);
        WTF::String result = sb.ToString();
        printf("[3] StringBuilder: \"%s\"\n", result.impl().c_str());
        if (result == "Bat_OS v3") {
            printf("    PASS\n"); passed++;
        }
    }

    // [4] WTF::AtomicString (interned strings for HTML tags)
    {
        WTF::AtomicString tag1("div");
        WTF::AtomicString tag2("div");
        WTF::AtomicString tag3("span");
        printf("[4] AtomicString: tag1=\"%s\" tag2=\"%s\" tag3=\"%s\"\n",
               tag1.Utf8(), tag2.Utf8(), tag3.Utf8());
        if (tag1 == tag2 && tag1 != tag3) {
            printf("    PASS\n"); passed++;
        }
    }

    // [5] WTF::Vector
    {
        WTF::Vector<int> v;
        v.push_back(10);
        v.push_back(20);
        v.push_back(30);
        int sum = 0;
        for (auto& x : v) sum += x;
        printf("[5] Vector: size=%zu sum=%d\n", v.size(), sum);
        if (v.size() == 3 && sum == 60) {
            printf("    PASS\n"); passed++;
        }
    }

    // [6] WTF::HashMap
    {
        WTF::HashMap<int, WTF::String> map;
        map[1] = WTF::String("one");
        map[2] = WTF::String("two");
        map[3] = WTF::String("three");
        printf("[6] HashMap: size=%zu map[2]=\"%s\"\n", map.size(), map[2].impl().c_str());
        if (map.size() == 3 && map[2] == "two") {
            printf("    PASS\n"); passed++;
        }
    }

    // [7] base::scoped_refptr (DOM tree ref counting)
    {
        struct TestNode : public base::RefCounted<TestNode> {
            int value;
            TestNode(int v) : value(v) {}
        };
        auto node = base::MakeRefCounted<TestNode>(42);
        printf("[7] RefCounted: value=%d hasOneRef=%d\n", node->value, node->HasOneRef());
        if (node->value == 42) {
            printf("    PASS\n"); passed++;
        }
    }

    // [8] base::raw_ptr (safe pointer wrapper)
    {
        int x = 99;
        base::raw_ptr<int> p(&x);
        printf("[8] raw_ptr: *p=%d\n", *p);
        if (*p == 99) {
            printf("    PASS\n"); passed++;
        }
    }

    printf("\n===================================\n");
    printf("Blink Foundation: %d/8 passed\n", passed);
    if (passed == 8) {
        printf("=== CHROMIUM FOUNDATION READY ===\n");
    }
    printf("===================================\n");

    exit(0);
}
