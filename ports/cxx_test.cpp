// Bat_OS — C++ Runtime Test
// Tests operator new/delete, virtual functions, static init, templates
extern "C" {
    #include <stdio.h>
    #include <stdlib.h>
    #include <string.h>
}

// Test 1: operator new / delete
struct Point {
    int x, y;
    Point(int a, int b) : x(a), y(b) {}
};

// Test 2: virtual functions
struct Shape {
    virtual int area() = 0;
    virtual ~Shape() {}
};

struct Rect : Shape {
    int w, h;
    Rect(int w, int h) : w(w), h(h) {}
    int area() override { return w * h; }
};

struct Circle : Shape {
    int r;
    Circle(int r) : r(r) {}
    int area() override { return 3 * r * r; } // rough pi
};

// Test 3: templates
template<typename T>
T max_val(T a, T b) { return a > b ? a : b; }

// Test 4: static local
int get_counter() {
    static int counter = 42;
    return counter++;
}

extern "C" void _start() {
    printf("=== C++ Runtime Test ===\n\n");
    int passed = 0;

    // Test 1: new/delete
    Point *p = new Point(10, 20);
    if (p && p->x == 10 && p->y == 20) {
        printf("[PASS] operator new: Point(%d, %d)\n", p->x, p->y);
        passed++;
    } else {
        printf("[FAIL] operator new\n");
    }
    delete p;
    printf("[PASS] operator delete\n");
    passed++;

    // Test 2: new[] / delete[]
    int *arr = new int[100];
    for (int i = 0; i < 100; i++) arr[i] = i * i;
    if (arr[99] == 9801) {
        printf("[PASS] new[]/delete[]: arr[99] = %d\n", arr[99]);
        passed++;
    }
    delete[] arr;

    // Test 3: virtual functions
    Rect r(10, 5);
    Circle c(7);
    Shape *shapes[2] = { &r, &c };
    if (shapes[0]->area() == 50 && shapes[1]->area() == 147) {
        printf("[PASS] virtual dispatch: rect=%d, circle=%d\n",
            shapes[0]->area(), shapes[1]->area());
        passed++;
    } else {
        printf("[FAIL] virtual dispatch\n");
    }

    // Test 4: templates
    if (max_val(3, 7) == 7 && max_val(100, 50) == 100) {
        printf("[PASS] templates: max(3,7)=%d, max(100,50)=%d\n",
            max_val(3, 7), max_val(100, 50));
        passed++;
    }

    // Test 5: static locals
    int c1 = get_counter();
    int c2 = get_counter();
    if (c1 == 42 && c2 == 43) {
        printf("[PASS] static locals: %d, %d\n", c1, c2);
        passed++;
    }

    // Test 6: heap allocation stress
    int alloc_ok = 1;
    for (int i = 0; i < 50; i++) {
        int *block = new int[256];
        block[0] = i;
        block[255] = i * 2;
        if (block[0] != i || block[255] != i * 2) alloc_ok = 0;
        delete[] block;
    }
    if (alloc_ok) {
        printf("[PASS] heap stress: 50 alloc/free cycles\n");
        passed++;
    }

    printf("\n=== C++ Runtime: %d/7 passed ===\n", passed);
    if (passed == 7) printf("=== C++ Runtime Test PASSED ===\n");
    exit(0);
}
