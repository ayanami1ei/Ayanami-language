#include "Lexer.h"

bool Lexer::isSpace(uint32_t c) {
    return c == ' ' || c == 9;
}

bool Lexer::isNewLine(uint32_t c) {
    return c == '\n' || c=='\r';
}

bool Lexer::isAlpha(uint32_t c) {
    return (c <= 'Z' && c >= 'A') || (c <= 'z' && c >= 'a');
}

bool Lexer::isChinese(uint32_t c) {
    return (c >= 0x4E00 && c <= 0x9FFF);
}

bool Lexer::isNumber(uint32_t c) {
    return '0' <= c && c <= '9';
}

uint32_t Lexer::advance() {
    if (pos < source.size()) {
        return source[pos++];
    }
    else {
        return '\0'; // 文件末尾返回空字符
    }
}

void Lexer::skipSpace() {
    ++pos;
}

Token Lexer::readIdentifierOrKeyword() {
    std::vector<uint32_t> word;
    while (isAlpha(peek()) || isNumber(peek()) || peek() == '_' || isChinese(peek())) {
        word.push_back(advance());
    }

    if (word == stringToUint32ts("input"))
        return { TokenType::INPUT, word, line };
    else if (word == stringToUint32ts("output"))
        return { TokenType::OUTPUT, word, line };
    else if (keywords.isKeyword(word))
        return { keywords.getEnum(word), word, line};
    else
        return { TokenType::IDENTIFIER, word, line };
}

Token Lexer::readNumber() {
    std::vector<uint32_t> num;
    bool hasDot = false;

    while (isNumber(peek()) || peek() == '.') {
        if (peek() == '.') {
            if (hasDot)
                break; // 第二个小数点就停止
            hasDot = true;
        }
        num.push_back(advance());
    }

    if (hasDot)
        return { TokenType::FLOAT_LITERAL, num, line };
    else
        return { TokenType::INT_LITERAL, num, line };
}

Token Lexer::readCharOrString() {
    std::vector<uint32_t> str;
    advance();
    while (peek()!='\'') {
        str.push_back(advance());
    }
    advance();

    return { TokenType::CHAR_LITERAL, str, line };
}

Token Lexer::readOperatorOrDelimiter() {
    uint32_t c = advance();
    std::vector<uint32_t> t;
    t.push_back(c);
    Keyword k;
    switch (c) {
    case '+': case '-': case '*': case '/': case '<': case '>': 
    case '(': case ')': case '{': case '}': case ',':case '[' :case ']':
        return { k.getEnum(t), stringToUint32ts(std::string(1, c)), line };
    case '=':
        if (source[pos] == '=') {
            t.push_back('=');
            advance();
        }
        return { k.getEnum(t), stringToUint32ts(std::string(1, c)), line };
    case '!':
        if (source[pos] == '=') {
            t.push_back('=');
            advance();
        }
        return { k.getEnum(t), stringToUint32ts(std::string(1, c)), line }; 
    case '&':
        if (source[pos] == '&') {
            t.push_back('&');
            advance();
        }
        return { k.getEnum(t), stringToUint32ts(std::string(1, c)), line };
    case '|':
        if (source[pos] == '|') {
            t.push_back('|');
            advance();
        }
        return { k.getEnum(t), stringToUint32ts(std::string(1, c)), line };

    default:
        throw std::runtime_error(
            "Unknown symbol: " + std::to_string(c) + " at line "
            + std::to_string(line) + "\n");
        std::cerr << "Unknown symbol: " << c << " at line " << line << "\n";
        return { TokenType::DELIMITER, stringToUint32ts(std::string(1, c)), line };
    }
}

uint32_t Lexer::peek() {
    return source[pos];
}

Lexer::Lexer(const std::vector<uint32_t> src) :source(src) {

}

std::vector<Token> Lexer::tokenize() {
    std::vector<Token> tokens;
    while (pos < source.size()) {
        uint32_t c = peek();

        if (isSpace(c)) {
            skipSpace();
        }
        else if (isNewLine(c)) {
            tokens.push_back({ TokenType::NEWLINE, stringToUint32ts("\\n"), line });
            advance();
            line++;
        }
        else if (isAlpha(c) || isChinese(c)) {
            tokens.push_back(readIdentifierOrKeyword());
        }
        else if (isNumber(c)) {
            tokens.push_back(readNumber());
        }
        else if (c == '\'' || c == '"') {
            tokens.push_back(readCharOrString());
        }
        else {
            tokens.push_back(readOperatorOrDelimiter());
        }
    }
    tokens.push_back({ TokenType::END_OF_FILE, stringToUint32ts(""), line });
    return tokens;
}
