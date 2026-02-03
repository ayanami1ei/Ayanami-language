#include <cmath>
#include "types.h"

/* forward decls for allocators used in macros */
extern "C" Object *alloc_int(const int value);
extern "C" Object *alloc_bool(const bool value);
extern "C" Object *alloc_float(const double value);
extern "C" Object *alloc_char(const char value);
extern "C" void runtime_debug_ref(int op, int slot_id, Object *obj);

#define UNION(a, b) a##b

#ifdef DRUNTIME_DEBUG
extern "C" void runtime_register(Object *obj);
extern "C" void runtime_unregister(Object *obj);
extern "C" int runtime_is_registered(Object *obj);
#endif

#define FREE_AS(type)                     \
    case VarType::type:                   \
        free((UNION(type, Object) *)obj); \
        break;

#ifdef DRUNTIME_DEBUG
#define alloc_fn(name, _type, TYPENAME)                                                                    \
    extern "C" Object *UNION(alloc_, name)(_type value)                                                    \
    {                                                                                                      \
        UNION(TYPENAME, Object) *ptr = (UNION(TYPENAME, Object) *)malloc(sizeof(UNION(TYPENAME, Object))); \
        if (ptr == NULL)                                                                                   \
        {                                                                                                  \
            return NULL;                                                                                   \
        }                                                                                                  \
        ptr->header.type = VarType::TYPENAME;                                                              \
        ptr->header.refcnt = 0;                                                                            \
        ptr->value = value;                                                                                \
        runtime_register((Object *)ptr);                                                                   \
        fprintf(stderr, "[runtime] alloc_%s value=%g ptr=%p\n", #name, (double)value, (void *)ptr);        \
        return (Object *)ptr;                                                                              \
    }
#else
#define alloc_fn(name, _type, TYPENAME)                                                                    \
    extern "C" Object *UNION(alloc_, name)(_type value)                                                    \
    {                                                                                                      \
        UNION(TYPENAME, Object) *ptr = (UNION(TYPENAME, Object) *)malloc(sizeof(UNION(TYPENAME, Object))); \
        if (ptr == NULL)                                                                                   \
        {                                                                                                  \
            return NULL;                                                                                   \
        }                                                                                                  \
        ptr->header.type = VarType::TYPENAME;                                                              \
        ptr->header.refcnt = 0;                                                                            \
        ptr->value = value;                                                                                \
        return (Object *)ptr;                                                                              \
    }
#endif

#define type_trans(ptr, type) \
    (type *)ptr;

/* helper: convert any Object* to a truthy 0/1 value (used by logical operators) */
extern "C" bool is_truth(const Object *obj)
{
    if (obj == NULL)
        return 0;
    switch (obj->type)
    {
    case VarType::Int:
    {
        IntObject *o = (IntObject *)obj;
        return (o->value != 0) ? 1 : 0;
    }
    case VarType::Float:
    {
        FloatObject *o = (FloatObject *)obj;
        return (fabs(o->value) > 1e-12) ? 1 : 0;
    }
    case VarType::Bool:
    {
        BoolObject *o = (BoolObject *)obj;
        return o->value ? 1 : 0;
    }
    case VarType::Char:
    {
        CharObject *o = (CharObject *)obj;
        return (o->value != 0) ? 1 : 0;
    }
    case VarType::String:
    {
        StringObject *o = (StringObject *)obj;
        return (o->len != 0) ? 1 : 0;
    }
    default:
        return 0;
    }
}

// C++ implementation of value_operation macro
#define value_operation(left, op, right)                                                   \
    do                                                                                     \
    {                                                                                      \
        VarType left_type = (left)->type;                                                  \
        VarType right_type = (right)->type;                                                \
        /* handle each type combination */                                                 \
        if (left_type == VarType::Int && right_type == VarType::Int)                       \
        {                                                                                  \
            IntObject *l = (IntObject *)(left);                                            \
            IntObject *r = (IntObject *)(right);                                           \
            /*printf("int and int\n");   */                                                \
            return alloc_int((l->value)op(r->value));                                      \
        }                                                                                  \
        else if (left_type == VarType::Int && right_type == VarType::Float)                \
        {                                                                                  \
            IntObject *l = (IntObject *)(left);                                            \
            FloatObject *r = (FloatObject *)(right);                                       \
            return alloc_float(((double)l->value)op(r->value));                            \
        }                                                                                  \
        else if (left_type == VarType::Int && right_type == VarType::Bool)                 \
        {                                                                                  \
            IntObject *l = (IntObject *)(left);                                            \
            BoolObject *r = (BoolObject *)(right);                                         \
            int rv = r->value ? 1 : 0;                                                     \
            return alloc_int((l->value)op rv);                                             \
        }                                                                                  \
        else if (left_type == VarType::Int && right_type == VarType::Char)                 \
        {                                                                                  \
            IntObject *l = (IntObject *)(left);                                            \
            CharObject *r = (CharObject *)(right);                                         \
            return alloc_char((char)((l->value)op(r->value)));                             \
        }                                                                                  \
        else if (left_type == VarType::Int && right_type == VarType::String)               \
        {                                                                                  \
            IntObject *l = (IntObject *)(left);                                            \
            StringObject *r = (StringObject *)(right);                                     \
            std::string left_str = std::to_string(l->value);                               \
            size_t new_len = r->len + left_str.size();                                     \
            char *buf = (char *)malloc(sizeof(char) * new_len);                            \
            if (!buf)                                                                      \
            {                                                                              \
                return NULL;                                                               \
            }                                                                              \
            memcpy(buf, left_str.data(), left_str.size());                                 \
            memcpy(buf + left_str.size(), r->data, r->len);                                \
            return alloc_string(buf, new_len);                                             \
        }                                                                                  \
        else if (left_type == VarType::Float && right_type == VarType::Int)                \
        {                                                                                  \
            FloatObject *l = (FloatObject *)(left);                                        \
            IntObject *r = (IntObject *)(right);                                           \
            return alloc_float((l->value)op((double)r->value));                            \
        }                                                                                  \
        else if (left_type == VarType::Float && right_type == VarType::Float)              \
        {                                                                                  \
            FloatObject *l = (FloatObject *)(left);                                        \
            FloatObject *r = (FloatObject *)(right);                                       \
            return alloc_float((l->value)op(r->value));                                    \
        }                                                                                  \
        else if (left_type == VarType::Float && right_type == VarType::Bool)               \
        {                                                                                  \
            FloatObject *l = (FloatObject *)(left);                                        \
            BoolObject *r = (BoolObject *)(right);                                         \
            double rv = r->value ? 1.0 : 0.0;                                              \
            return alloc_float((l->value)op rv);                                           \
        }                                                                                  \
        else if (left_type == VarType::Float && right_type == VarType::Char)               \
        {                                                                                  \
            FloatObject *l = (FloatObject *)(left);                                        \
            CharObject *r = (CharObject *)(right);                                         \
            return alloc_float((l->value)op((double)r->value));                            \
        }                                                                                  \
        else if (left_type == VarType::Float && right_type == VarType::String)             \
        {                                                                                  \
            FloatObject *l = (FloatObject *)(left);                                        \
            StringObject *r = (StringObject *)(right);                                     \
            std::string left_str = std::to_string(l->value);                               \
            size_t new_len = r->len + left_str.size();                                     \
            char *buf = (char *)malloc(sizeof(char) * new_len);                            \
            if (!buf)                                                                      \
            {                                                                              \
                return NULL;                                                               \
            }                                                                              \
            memcpy(buf, left_str.data(), left_str.size());                                 \
            memcpy(buf + left_str.size(), r->data, r->len);                                \
            return alloc_string(buf, new_len);                                             \
        }                                                                                  \
        else if (left_type == VarType::Bool && right_type == VarType::Int)                 \
        {                                                                                  \
            BoolObject *l = (BoolObject *)(left);                                          \
            IntObject *r = (IntObject *)(right);                                           \
            int lv = l->value ? 1 : 0;                                                     \
            return alloc_int((lv)op(r->value));                                            \
        }                                                                                  \
        else if (left_type == VarType::Bool && right_type == VarType::Float)               \
        {                                                                                  \
            BoolObject *l = (BoolObject *)(left);                                          \
            FloatObject *r = (FloatObject *)(right);                                       \
            double lv = l->value ? 1.0 : 0.0;                                              \
            return alloc_float((lv)op(r->value));                                          \
        }                                                                                  \
        else if (left_type == VarType::Bool && right_type == VarType::Bool)                \
        {                                                                                  \
            BoolObject *l = (BoolObject *)(left);                                          \
            BoolObject *r = (BoolObject *)(right);                                         \
            int lv = l->value ? 1 : 0;                                                     \
            int rv = r->value ? 1 : 0;                                                     \
            return alloc_int((lv)op(rv));                                                  \
        }                                                                                  \
        else if (left_type == VarType::Bool && right_type == VarType::Char)                \
        {                                                                                  \
            BoolObject *l = (BoolObject *)(left);                                          \
            CharObject *r = (CharObject *)(right);                                         \
            int lv = l->value ? 1 : 0;                                                     \
            return alloc_char((char)((lv)op(r->value)));                                   \
        }                                                                                  \
        else if (left_type == VarType::Bool && right_type == VarType::String)              \
        {                                                                                  \
            BoolObject *l = (BoolObject *)(left);                                          \
            StringObject *r = (StringObject *)(right);                                     \
            std::string left_str = l->value ? std::string("true") : std::string("false");  \
            size_t new_len = r->len + left_str.size();                                     \
            char *buf = (char *)malloc(sizeof(char) * new_len);                            \
            if (!buf)                                                                      \
            {                                                                              \
                return NULL;                                                               \
            }                                                                              \
            memcpy(buf, left_str.data(), left_str.size());                                 \
            memcpy(buf + left_str.size(), r->data, r->len);                                \
            return alloc_string(buf, new_len);                                             \
        }                                                                                  \
        else if (left_type == VarType::Char && right_type == VarType::Int)                 \
        {                                                                                  \
            CharObject *l = (CharObject *)(left);                                          \
            IntObject *r = (IntObject *)(right);                                           \
            return alloc_char((char)((l->value)op(r->value)));                             \
        }                                                                                  \
        else if (left_type == VarType::Char && right_type == VarType::Float)               \
        {                                                                                  \
            CharObject *l = (CharObject *)(left);                                          \
            FloatObject *r = (FloatObject *)(right);                                       \
            return alloc_float(((double)l->value)op(r->value));                            \
        }                                                                                  \
        else if (left_type == VarType::Char && right_type == VarType::Bool)                \
        {                                                                                  \
            CharObject *l = (CharObject *)(left);                                          \
            BoolObject *r = (BoolObject *)(right);                                         \
            int rv = r->value ? 1 : 0;                                                     \
            return alloc_char((char)((l->value)op rv));                                    \
        }                                                                                  \
        else if (left_type == VarType::Char && right_type == VarType::Char)                \
        {                                                                                  \
            CharObject *l = (CharObject *)(left);                                          \
            CharObject *r = (CharObject *)(right);                                         \
            return alloc_char((char)((l->value)op(r->value)));                             \
        }                                                                                  \
        else if (left_type == VarType::Char && right_type == VarType::String)              \
        {                                                                                  \
            CharObject *l = (CharObject *)(left);                                          \
            StringObject *r = (StringObject *)(right);                                     \
            std::string left_str(1, (char)l->value);                                       \
            size_t new_len = r->len + 1;                                                   \
            char *buf = (char *)malloc(sizeof(char) * new_len);                            \
            if (!buf)                                                                      \
            {                                                                              \
                return NULL;                                                               \
            }                                                                              \
            buf[0] = l->value;                                                             \
            memcpy(buf + 1, r->data, r->len);                                              \
            return alloc_string(buf, new_len);                                             \
        }                                                                                  \
        else if (left_type == VarType::String && right_type == VarType::Int)               \
        {                                                                                  \
            StringObject *l = (StringObject *)(left);                                      \
            IntObject *r = (IntObject *)(right);                                           \
            std::string right_str = std::to_string(r->value);                              \
            size_t new_len = l->len + right_str.size();                                    \
            char *buf = (char *)malloc(sizeof(char) * new_len);                            \
            if (!buf)                                                                      \
            {                                                                              \
                return NULL;                                                               \
            }                                                                              \
            memcpy(buf, l->data, l->len);                                                  \
            memcpy(buf + l->len, right_str.data(), right_str.size());                      \
            return alloc_string(buf, new_len);                                             \
        }                                                                                  \
        else if (left_type == VarType::String && right_type == VarType::Float)             \
        {                                                                                  \
            StringObject *l = (StringObject *)(left);                                      \
            FloatObject *r = (FloatObject *)(right);                                       \
            std::string right_str = std::to_string(r->value);                              \
            size_t new_len = l->len + right_str.size();                                    \
            char *buf = (char *)malloc(sizeof(char) * new_len);                            \
            if (!buf)                                                                      \
            {                                                                              \
                return NULL;                                                               \
            }                                                                              \
            memcpy(buf, l->data, l->len);                                                  \
            memcpy(buf + l->len, right_str.data(), right_str.size());                      \
            return alloc_string(buf, new_len);                                             \
        }                                                                                  \
        else if (left_type == VarType::String && right_type == VarType::Bool)              \
        {                                                                                  \
            StringObject *l = (StringObject *)(left);                                      \
            BoolObject *r = (BoolObject *)(right);                                         \
            std::string right_str = r->value ? std::string("true") : std::string("false"); \
            size_t new_len = l->len + right_str.size();                                    \
            char *buf = (char *)malloc(sizeof(char) * new_len);                            \
            if (!buf)                                                                      \
            {                                                                              \
                return NULL;                                                               \
            }                                                                              \
            memcpy(buf, l->data, l->len);                                                  \
            memcpy(buf + l->len, right_str.data(), right_str.size());                      \
            return alloc_string(buf, new_len);                                             \
        }                                                                                  \
        else if (left_type == VarType::String && right_type == VarType::Char)              \
        {                                                                                  \
            StringObject *l = (StringObject *)(left);                                      \
            CharObject *r = (CharObject *)(right);                                         \
            std::string right_str(1, (char)r->value);                                      \
            size_t new_len = l->len + 1;                                                   \
            char *buf = (char *)malloc(sizeof(char) * new_len);                            \
            if (!buf)                                                                      \
            {                                                                              \
                return NULL;                                                               \
            }                                                                              \
            memcpy(buf, l->data, l->len);                                                  \
            buf[l->len] = r->value;                                                        \
            return alloc_string(buf, new_len);                                             \
        }                                                                                  \
        else if (left_type == VarType::String && right_type == VarType::String)            \
        {                                                                                  \
            StringObject *l = (StringObject *)(left);                                      \
            StringObject *r = (StringObject *)(right);                                     \
            size_t new_len = l->len + r->len;                                              \
            char *buf = (char *)malloc(sizeof(char) * new_len);                            \
            if (!buf)                                                                      \
            {                                                                              \
                return NULL;                                                               \
            }                                                                              \
            memcpy(buf, l->data, l->len);                                                  \
            memcpy(buf + l->len, r->data, r->len);                                         \
            return alloc_string(buf, new_len);                                             \
        }                                                                                  \
        else                                                                               \
        {                                                                                  \
            const char *msg = "unknown type\n";                                            \
            printf("left type: %d, right type: %d\n", (int)left->type, (int)right->type);  \
            char *buf = (char *)malloc(sizeof(char) * strlen(msg));                        \
            if (!buf)                                                                      \
            {                                                                              \
                return NULL;                                                               \
            }                                                                              \
            memcpy(buf, msg, strlen(msg));                                                 \
            return alloc_string(buf, strlen(msg));                                         \
        }                                                                                  \
    } while (0)

#define logic_operation(left, op, right)                                                  \
    do                                                                                    \
    {                                                                                     \
        VarType left_type = (left)->type;                                                 \
        VarType right_type = (right)->type;                                               \
        /* handle each type combination */                                                \
        if (left_type == VarType::Int && right_type == VarType::Int)                      \
        {                                                                                 \
            IntObject *l = (IntObject *)(left);                                           \
            IntObject *r = (IntObject *)(right);                                          \
            /*printf("int and int\n");   */                                               \
            return alloc_bool((l->value)op(r->value));                                    \
        }                                                                                 \
        else if (left_type == VarType::Int && right_type == VarType::Float)               \
        {                                                                                 \
            IntObject *l = (IntObject *)(left);                                           \
            FloatObject *r = (FloatObject *)(right);                                      \
            return alloc_bool(((double)l->value)op(r->value));                            \
        }                                                                                 \
        else if (left_type == VarType::Int && right_type == VarType::Bool)                \
        {                                                                                 \
            IntObject *l = (IntObject *)(left);                                           \
            BoolObject *r = (BoolObject *)(right);                                        \
            int rv = r->value ? 1 : 0;                                                    \
            return alloc_bool((l->value)op rv);                                           \
        }                                                                                 \
        else if (left_type == VarType::Int && right_type == VarType::Char)                \
        {                                                                                 \
            IntObject *l = (IntObject *)(left);                                           \
            CharObject *r = (CharObject *)(right);                                        \
            return alloc_bool(((l->value)op(int)(r->value)));                             \
        }                                                                                 \
        else if (left_type == VarType::Int && right_type == VarType::String)              \
        {                                                                                 \
            IntObject *l = (IntObject *)(left);                                           \
            StringObject *r = (StringObject *)(right);                                    \
            return alloc_bool((l->value)op(r->len));                                      \
        }                                                                                 \
        else if (left_type == VarType::Float && right_type == VarType::Int)               \
        {                                                                                 \
            FloatObject *l = (FloatObject *)(left);                                       \
            IntObject *r = (IntObject *)(right);                                          \
            return alloc_bool((l->value)op((double)r->value));                            \
        }                                                                                 \
        else if (left_type == VarType::Float && right_type == VarType::Float)             \
        {                                                                                 \
            FloatObject *l = (FloatObject *)(left);                                       \
            FloatObject *r = (FloatObject *)(right);                                      \
            return alloc_bool((l->value)op(r->value));                                    \
        }                                                                                 \
        else if (left_type == VarType::Float && right_type == VarType::Bool)              \
        {                                                                                 \
            FloatObject *l = (FloatObject *)(left);                                       \
            BoolObject *r = (BoolObject *)(right);                                        \
            double rv = r->value ? 1.0 : 0.0;                                             \
            return alloc_bool((l->value)op rv);                                           \
        }                                                                                 \
        else if (left_type == VarType::Float && right_type == VarType::Char)              \
        {                                                                                 \
            FloatObject *l = (FloatObject *)(left);                                       \
            CharObject *r = (CharObject *)(right);                                        \
            return alloc_bool((l->value)op((double)r->value));                            \
        }                                                                                 \
        else if (left_type == VarType::Float && right_type == VarType::String)            \
        {                                                                                 \
            FloatObject *l = (FloatObject *)(left);                                       \
            StringObject *r = (StringObject *)(right);                                    \
            return alloc_bool((l->value)op(r->len));                                      \
        }                                                                                 \
        else if (left_type == VarType::Bool && right_type == VarType::Int)                \
        {                                                                                 \
            BoolObject *l = (BoolObject *)(left);                                         \
            IntObject *r = (IntObject *)(right);                                          \
            int lv = l->value ? 1 : 0;                                                    \
            return alloc_bool((lv)op(r->value));                                          \
        }                                                                                 \
        else if (left_type == VarType::Bool && right_type == VarType::Float)              \
        {                                                                                 \
            BoolObject *l = (BoolObject *)(left);                                         \
            FloatObject *r = (FloatObject *)(right);                                      \
            double lv = l->value ? 1.0 : 0.0;                                             \
            return alloc_bool((lv)op(r->value));                                          \
        }                                                                                 \
        else if (left_type == VarType::Bool && right_type == VarType::Bool)               \
        {                                                                                 \
            BoolObject *l = (BoolObject *)(left);                                         \
            BoolObject *r = (BoolObject *)(right);                                        \
            int lv = l->value ? 1 : 0;                                                    \
            int rv = r->value ? 1 : 0;                                                    \
            return alloc_bool((lv)op(rv));                                                \
        }                                                                                 \
        else if (left_type == VarType::Bool && right_type == VarType::Char)               \
        {                                                                                 \
            BoolObject *l = (BoolObject *)(left);                                         \
            CharObject *r = (CharObject *)(right);                                        \
            int lv = l->value ? 1 : 0;                                                    \
            return alloc_bool((char)((lv)op(r->value)));                                  \
        }                                                                                 \
        else if (left_type == VarType::Bool && right_type == VarType::String)             \
        {                                                                                 \
            BoolObject *l = (BoolObject *)(left);                                         \
            StringObject *r = (StringObject *)(right);                                    \
            return alloc_bool((l->value)op(r->len));                                      \
        }                                                                                 \
        else if (left_type == VarType::Char && right_type == VarType::Int)                \
        {                                                                                 \
            CharObject *l = (CharObject *)(left);                                         \
            IntObject *r = (IntObject *)(right);                                          \
            return alloc_bool((char)((l->value)op(r->value)));                            \
        }                                                                                 \
        else if (left_type == VarType::Char && right_type == VarType::Float)              \
        {                                                                                 \
            CharObject *l = (CharObject *)(left);                                         \
            FloatObject *r = (FloatObject *)(right);                                      \
            return alloc_bool(((double)l->value)op(r->value));                            \
        }                                                                                 \
        else if (left_type == VarType::Char && right_type == VarType::Bool)               \
        {                                                                                 \
            CharObject *l = (CharObject *)(left);                                         \
            BoolObject *r = (BoolObject *)(right);                                        \
            int rv = r->value ? 1 : 0;                                                    \
            return alloc_bool((char)((l->value)op rv));                                   \
        }                                                                                 \
        else if (left_type == VarType::Char && right_type == VarType::Char)               \
        {                                                                                 \
            CharObject *l = (CharObject *)(left);                                         \
            CharObject *r = (CharObject *)(right);                                        \
            return alloc_bool((char)((l->value)op(r->value)));                            \
        }                                                                                 \
        else if (left_type == VarType::Char && right_type == VarType::String)             \
        {                                                                                 \
            CharObject *l = (CharObject *)(left);                                         \
            StringObject *r = (StringObject *)(right);                                    \
            return alloc_bool((l->value)op(r->len));                                      \
        }                                                                                 \
        else if (left_type == VarType::String && right_type == VarType::Int)              \
        {                                                                                 \
            StringObject *l = (StringObject *)(left);                                     \
            IntObject *r = (IntObject *)(right);                                          \
            return alloc_bool((l->len)op(r->value));                                      \
        }                                                                                 \
        else if (left_type == VarType::String && right_type == VarType::Float)            \
        {                                                                                 \
            StringObject *l = (StringObject *)(left);                                     \
            FloatObject *r = (FloatObject *)(right);                                      \
            return alloc_bool((l->len)op(r->value));                                      \
        }                                                                                 \
        else if (left_type == VarType::String && right_type == VarType::Bool)             \
        {                                                                                 \
            StringObject *l = (StringObject *)(left);                                     \
            BoolObject *r = (BoolObject *)(right);                                        \
            return alloc_bool((l->len)op(r->value));                                      \
        }                                                                                 \
        else if (left_type == VarType::String && right_type == VarType::Char)             \
        {                                                                                 \
            StringObject *l = (StringObject *)(left);                                     \
            CharObject *r = (CharObject *)(right);                                        \
            return alloc_bool((l->len)op(r->value));                                      \
        }                                                                                 \
        else if (left_type == VarType::String && right_type == VarType::String)           \
        {                                                                                 \
            StringObject *l = (StringObject *)(left);                                     \
            StringObject *r = (StringObject *)(right);                                    \
            return alloc_bool((l->len)op(r->len));                                        \
        }                                                                                 \
        else                                                                              \
        {                                                                                 \
            const char *msg = "unknown type\n";                                           \
            printf("left type: %d, right type: %d\n", (int)left->type, (int)right->type); \
            char *buf = (char *)malloc(sizeof(char) * strlen(msg));                       \
            if (!buf)                                                                     \
            {                                                                             \
                return NULL;                                                              \
            }                                                                             \
            memcpy(buf, msg, strlen(msg));                                                \
            return alloc_string(buf, strlen(msg));                                        \
        }                                                                                 \
    } while (0)

#define bin_op_inner(left, op, right, type) \
    UNION(type, _operation)(left, op, right)

#define bin_operation(name, left, op, right, type)                                       \
    extern "C" Object *name(const Object *UNION(_, left), const Object *UNION(_, right)) \
    {                                                                                    \
        if (UNION(_, left) == NULL || UNION(_, right) == NULL)                           \
        {                                                                                \
            return NULL;                                                                 \
        }                                                                                \
        bin_op_inner(UNION(_, left), op, UNION(_, right), type);                         \
    }
