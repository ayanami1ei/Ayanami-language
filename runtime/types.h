#ifndef AYANAMI_CPP_TYPES_H
#define AYANAMI_CPP_TYPES_H

#define DEBUG

enum class VarType
{
    Int=1,
    Float=2,
    Bool=3,
    Char=4,
    String=5,
    Array=6,
    Unknown=7,
};

struct Object
{
    VarType type;
    int refcnt;
};

struct IntObject
{
    Object header;
    int value;
};

struct FloatObject
{
    Object header;
    double value;
};

struct CharObject
{
    Object header;
    char value;
};

struct BoolObject
{
    Object header;
    bool value;
};

struct ArrayObject
{
    Object header;
    int len;
    Object **data;
};

struct StringObject
{
    Object header;
    int len;
    char *data;
};

#endif // AYANAMI_CPP_TYPES_H