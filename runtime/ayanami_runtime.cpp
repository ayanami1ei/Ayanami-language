#include "defines.h"
#include "types.h"
#include <stdio.h>
#include <stdlib.h>
#include <string>
#include <string.h>
#include <vector>

//#define DEBUG

// g++ -g -O0 -std=c++17 -I c -c ayanami_runtime.cpp -o ayanami_runtime.o && ar rcs libayanami_runtime.a ayanami_runtime.o
// g++ -g -O3 -std=c++17 -I c -c ayanami_runtime.cpp -o ayanami_runtime.o && ar rcs libayanami_runtime.a ayanami_runtime.o

extern "C" Object *alloc_string(char *value, int len);

extern "C" void err(Object *obj)
{
    #ifdef DEBUG
    fprintf(stderr, "[runtime] err called obj=%p\n", (void *)obj);
    #endif
    if (obj == NULL)
    {
        #ifdef DEBUG
        fprintf(stderr, "[runtime] err: NULL obj\n");
        #endif
        return;
    }
    StringObject *s = (StringObject *)obj;
        #ifdef DEBUG
    if (s->data != NULL)
    {
        fprintf(stderr, "%s\n", s->data);
        fflush(stderr);
    }
    else
    {
        fprintf(stderr, "[runtime] err: string->data NULL\n");
    }
        #endif

    exit(-1);
}

extern "C" void write(Object *obj)
{
    if (obj==NULL){
        Object *str_err = alloc_string("illegal str", strlen("illegal str"));
        err(str_err);
    }

    StringObject *str_obj;
    std::string str;

    switch (obj->type)
    {
    case VarType::Int:
        str = std::to_string(((IntObject *)obj)->value);
        str_obj = (StringObject *)alloc_string((char *)str.c_str(), str.size());
        break;
    case VarType::Float:
        str = std::to_string(((FloatObject *)obj)->value);
        str_obj = (StringObject *)alloc_string((char *)str.c_str(), str.size());
        break;
    case VarType::Bool:
        str = ((BoolObject *)obj)->value ? "true" : "false";
        str_obj = (StringObject *)alloc_string((char *)str.c_str(), str.size());
        break;
    case VarType::Char:
        str = std::to_string(((CharObject *)obj)->value);
        str_obj = (StringObject *)alloc_string((char *)str.c_str(), str.size());
        break;
    case VarType::String:
        str_obj = (StringObject *)obj;
        break;
    default:
        str_obj = (StringObject *)alloc_string("", 0);
    }

    printf("%s", (char *)(((StringObject *)str_obj)->data));
    fflush(stdout);
}

extern "C" bool is_type(int ty, Object *obj)
{
    #ifdef DEBUG
    // quick integrity scan to detect first corruption earlier
    fprintf(stderr, "[runtime] is_type called ty=%d obj=%p\n", ty, (void *)obj);
    #endif
    if (ty < 1 || ty > 6)
    {
        #ifdef DEBUG
        fprintf(stderr, "[runtime] is_type: invalid ty=%d\n", ty);
        #endif
        return false;
    }
    VarType type = static_cast<VarType>(ty);
    bool res = (obj != NULL) && (type == obj->type);
    #ifdef DEBUG
    fprintf(stderr, "[runtime] is_type result=%d\n", res);
    #endif
    return res;
}

extern "C" int get_int_value(Object *obj)
{
    switch (obj->type)
    {
    case VarType::Int:
        return ((IntObject *)obj)->value;
    case VarType::Float:
        return ((FloatObject *)obj)->value;
    case VarType::Bool:
        return ((BoolObject *)obj)->value;
    case VarType::Char:
        return ((CharObject *)obj)->value;
    }

    abort();
}

extern "C" void del_obj(Object *obj)
{
    if (obj == NULL)
    {
        return;
    }

    int t = (int)obj->type;
    if (t < 1 || t > 6)
    {
        #ifdef DEBUG
        fprintf(stderr, "[runtime] del_obj: invalid header for obj=%p type=%d refcnt=%d\n", (void *)obj, t, obj->refcnt);
        // Dump first bytes to help diagnose pointer/value confusion
        unsigned char *p = (unsigned char *)obj;
        fprintf(stderr, "[runtime] dump bytes:");
        for (int i = 0; i < 32; ++i)
        {
            fprintf(stderr, " %02x", (unsigned int)p[i]);
        }
        fprintf(stderr, "\n");
        fflush(stderr);
        #endif
        abort();
    }

    #ifdef DEBUG
    fprintf(stderr, "[runtime] del_obj called obj=%p type=%d refcnt=%d\n", (void *)obj, t, obj->refcnt);
    #endif

    switch (obj->type)
    {
    case VarType::Int:
    {
        free((IntObject *)obj);
        break;
    }
    case VarType::Float:
    {
        free((FloatObject *)obj);
        break;
    }
    case VarType::Char:
    {
        free((CharObject *)obj);
        break;
    }
    case VarType::Bool:
    {
        free((BoolObject *)obj);
        break;
    }
    case VarType::String:
    {
        // free internal buffer first, then the string object
        StringObject *s = (StringObject *)obj;
        free(s);
        break;
    }
    }
}

extern "C" void dec_ref(Object *obj)
{
    if (obj == NULL)
    {
        return;
    }

    #ifdef DEBUG 
    if ((*obj).refcnt <= 0)
    {
        fprintf(stderr, "[runtime] dec_ref: warning obj=%p had non-positive refcnt=%d\n", (void *)obj, (int)(*obj).refcnt);
    }
    #endif
    (*obj).refcnt -= 1;
    #ifdef DEBUG
    fprintf(stderr, "[runtime] dec_ref obj=%p new_refcnt=%d\n", (void *)obj, (int)(*obj).refcnt);
#endif

    if ((*obj).refcnt <= 0)
    {
        #ifdef DEBUG
        fprintf(stderr, "[runtime] refcnt reached zero for obj=%p, calling del_obj\n", (void *)obj);
        #endif
        del_obj(obj);
    }
}

extern "C" void inc_ref(Object *obj)
{
    if (obj == NULL)
    {
        return;
    }

    #ifdef DEBUG
    if ((*obj).refcnt < 0)
    {
        fprintf(stderr, "[runtime] inc_ref: warning obj=%p had negative refcnt=%d\n", (void *)obj, (int)(*obj).refcnt);
    }
    #endif
    (*obj).refcnt += 1;

    #ifdef DEBUG
    fprintf(stderr, "[runtime] inc_ref obj=%p new_refcnt=%d\n", (void *)obj, (int)(*obj).refcnt);
    #endif
}

alloc_fn(int, const int, Int)
    alloc_fn(bool, const bool, Bool)
        alloc_fn(float, const double, Float)
            alloc_fn(char, const char, Char)

                extern "C" Object *alloc_string(char *value, int len)
{
    StringObject *ptr = (StringObject *)malloc(sizeof(StringObject));
    if (ptr == NULL)
    {
        return NULL;
    }

    // Allocate a buffer and copy the provided data, ensuring a NUL terminator.
    // Many call sites construct buffers without adding a terminating '\0',
    // and callers sometimes pass heap or global pointers. To make string
    // handling robust, always copy `len` bytes and append a '\0'. This avoids
    // printf("%s") reading past the allocation.
    char *buf = (char *)malloc((size_t)len + 1);
    if (buf == NULL)
    {
        free(ptr);
        return NULL;
    }
    if (value != NULL && len > 0)
    {
        memcpy(buf, value, (size_t)len);
    }
    buf[len] = '\0';

    /* initialize header */
    ptr->header.type = VarType::String;
    ptr->header.refcnt = 1;
    ptr->data = buf;
    ptr->len = len;

    #ifdef DEBUG
    fprintf(stderr, "[runtime] alloc_string called len=%d ptr=%p data=%p\n", len, (void *)ptr, (void *)ptr->data);
    #endif

    return (Object *)ptr;
}

bin_operation(add, left, +, right, value)
    bin_operation(sub, left, -, right, value)
        bin_operation(mul, left, *, right, value)
            bin_operation(div_op, left, /, right, value)
                bin_operation(and_op, left, &&, right, logic)
                    bin_operation(or_op, left, ||, right, logic)
                        bin_operation(equal, left, ==, right, logic)
                            bin_operation(greater, left, >, right, logic)
                                bin_operation(less, left, <, right, logic)
                                    bin_operation(greater_equal, left, >=, right, logic)
                                        bin_operation(less_equal, left, <=, right, logic)

                                            extern "C" Object *not_op(Object *obj)
{
    return alloc_bool(!is_truth(obj));
}