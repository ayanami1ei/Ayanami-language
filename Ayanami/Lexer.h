#pragma once
#include<vector>
#include <string>
#include <unordered_set>
#include <iostream>
#include "utf8.h"
#ifndef UTIL_H
#define UTIL_H
#include"util.h"
#endif



class Lexer {
private:
    Keyword keywords;
    std::vector<uint32_t> source;
    size_t pos = 0;
    int line = 1;

    bool isSpace(uint32_t c);

    bool isNewLine(uint32_t c);

    bool isAlpha(uint32_t c);

    bool isChinese(uint32_t c);

    bool isNumber(uint32_t c);

    uint32_t advance();

    void skipSpace();

    Token readNumber();

    Token readCharOrString();

    Token readOperatorOrDelimiter();

    Token readIdentifierOrKeyword();

    uint32_t peek();

public:
    Lexer(const std::vector<uint32_t> src);

    std::vector<Token> tokenize();
};