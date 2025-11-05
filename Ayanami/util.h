#pragma once
#include<vector>
#include <string>
#include <unordered_set>
#include <iostream>
#include "utf8.h"

#ifndef UTIL_H
#define UTIL_H
#endif

inline std::vector<uint32_t> stringToUint32ts(const std::string s) {
    std::vector<uint32_t> result;
    auto it = s.begin();
    while (it != s.end())
        result.push_back(utf8::next(it, s.end()));
    return result;
}

inline std::string uint32tsToString(const std::vector<uint32_t>& utf32) {
    std::string utf8;
    for (uint32_t codepoint : utf32) {
        if (codepoint <= 0x7F) {
            utf8.push_back(static_cast<char>(codepoint));
        }
        else if (codepoint <= 0x7FF) {
            utf8.push_back(static_cast<char>(0xC0 | (codepoint >> 6)));
            utf8.push_back(static_cast<char>(0x80 | (codepoint & 0x3F)));
        }
        else if (codepoint <= 0xFFFF) {
            utf8.push_back(static_cast<char>(0xE0 | (codepoint >> 12)));
            utf8.push_back(static_cast<char>(0x80 | ((codepoint >> 6) & 0x3F)));
            utf8.push_back(static_cast<char>(0x80 | (codepoint & 0x3F)));
        }
        else if (codepoint <= 0x10FFFF) {
            utf8.push_back(static_cast<char>(0xF0 | (codepoint >> 18)));
            utf8.push_back(static_cast<char>(0x80 | ((codepoint >> 12) & 0x3F)));
            utf8.push_back(static_cast<char>(0x80 | ((codepoint >> 6) & 0x3F)));
            utf8.push_back(static_cast<char>(0x80 | (codepoint & 0x3F)));
        }
        else {
            // 非法的 Unicode 码点，替换为 U+FFFD
            utf8 += "\xEF\xBF\xBD";
        }
    }
    return utf8;
}


enum class TokenType {
    IDENTIFIER,     // 标识符
    BOOL_LITERAL,
    INT_LITERAL,
    FLOAT_LITERAL,
    CHAR_LITERAL,
    KEYWORD,        // 关键字(fn, if, for, return...)
    FN,
    IF,
    FOR,
    WHILE,
    DO,
    IN,
    CONTINUE,
    BREAK,
    RETURN,
    REF,
    CONST,
    BOOL,
    INT,
    FLOAT,
    CHAR,
    VOID,
    TRUE,
    FALSE,
    OPERATOR,       // + - * / =
    ADD,
    SUB,
    MUL,
    DEV,
    EQUAL,
    OR,
    AND,
    EQUAL_EQUAL,
    NOT_EQUAL,
    LESS,
    GREATER,
    DELIMITER,      // , ( ) { }
    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,
    COMMA,
    LSQUARE,
    RSQUARE,
    NEWLINE,        // 换行
    END_OF_FILE,     // 文件结束
    ARR_BOOL,
    ARR_INT,
    ARR_FLOAT,
    ARR_CHAR,
    ARR_ARR,
    UNKNOWN,
    INPUT,
    OUTPUT,
};

struct Token {
    TokenType type;
    std::vector<uint32_t> lexeme;
    int line;

    friend std::ostream& operator<<(std::ostream& out, Token& a) {
        out << "[";
        switch (a.type) {
        case TokenType::IDENTIFIER:    // 标识符
            out << "标识符";
            break;
        case TokenType::FN:        // 关键字(fn, if, for, return...)
        case TokenType::IF:
        case TokenType::FOR:
        case TokenType::WHILE:
        case TokenType::DO:
        case TokenType::RETURN:
        case TokenType::REF:
        case TokenType::CONST:
        case TokenType::BOOL:
        case TokenType::INT:
        case TokenType::FLOAT:
        case TokenType::CHAR:
        case TokenType::TRUE:
        case TokenType::FALSE:
            out << "关键字";
            break;
        case TokenType::INT_LITERAL:   // 整数字面值
            out << "整数字面值";
            break;
        case TokenType::FLOAT_LITERAL:  // 浮点数字面值
            out << "浮点数字面值";
            break;
        case TokenType::CHAR_LITERAL:   // 字符字面值
            out << "字符字面值";
            break;
        case TokenType::OPERATOR:       // + - * / =
        case TokenType::ADD:
        case TokenType::SUB:
        case TokenType::MUL:
        case TokenType::DEV:
        case TokenType::EQUAL:
        case TokenType::OR:
        case TokenType::AND:
        case TokenType::EQUAL_EQUAL:
        case TokenType::NOT_EQUAL:
        case TokenType::LESS:
        case TokenType::GREATER:
            out << "操作符";
            break;
        case TokenType::DELIMITER:      // , ( ) { }
        case TokenType::LPAREN:
        case TokenType::RPAREN:
        case TokenType::LBRACE:
        case TokenType::RBRACE:
        case TokenType::COMMA:
            out << "括号逗号";
            break;
        case TokenType::NEWLINE:        // 换行
            out << "换行";
            break;
        case TokenType::END_OF_FILE:     // 文件结束
            out << "文件结束";
            break;
        }
        out << ",";

        std::string utf8_str = uint32tsToString(a.lexeme);
        out << utf8_str << "]";
        return out;
    }
};

class Keyword {
private:
    std::vector<std::vector<uint32_t>> keywords;
public:
    Keyword(){
        std::vector<std::string> rawKeywords = {
         "fn", "if", "for", "while", "do","in","continue","break", "return",
         "ref", "const", "bool", "int", "float", "char","void","true","false",
         "+","-","*","/","=","||","&&","==","!=","<",">",
         "(",")","{","}",",","[","]"
        };  

        for (const auto& kw : rawKeywords) {
           keywords.push_back(stringToUint32ts(kw));
        }
    }

    bool isKeyword(const std::vector<uint32_t>& check) {
        for (auto& kw : keywords) {
            if (kw == check) {
                return true;
                break;
            }
        }

        return false;
    }

    TokenType getEnum(std::vector<uint32_t>& word) {
        int i = 0;
        for (i = 0; i < keywords.size(); i++) {
            if (word == keywords[i])
                break;
        }


        if (i < 18)
            return (TokenType)((int)TokenType::KEYWORD + i+1);
        else if (i < 29)
            return (TokenType)((int)TokenType::KEYWORD + i+2);
        else 
            return (TokenType)((int)TokenType::KEYWORD + i+3);
    }
};

struct Param {
    TokenType type;
    std::vector<uint32_t> name;
    bool isRef;
};


