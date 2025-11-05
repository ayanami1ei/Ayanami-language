#pragma once
#ifndef ASTNODE_H
#define ASTNODE_H
#include"ASTNode.h"
#endif

#ifndef UTIL_H
#define UTIL_H
#include"util.h"
#endif

#ifndef PAESER_H
#define PAESER_H
#endif


class Parser {
private:
	std::vector<Token> tokens;
	size_t pos;

    Token peek() const { 
        return tokens[pos];
    }

    Token advance() {
        return tokens[pos++]; 
    }

    Token consume(TokenType type, const std::string& msg) {
        if (check(type)) 
            return advance();
        throw std::runtime_error("Parse error: " + msg);
    }

    bool match(TokenType t) {
        TokenType p = peek().type;
        if (peek().type == t) { 
            advance();
            return true;
        }
        return false;
    }

    bool check(TokenType t) {
        return peek().type == t;
    }

    bool isAtEnd() const {
        return pos >= tokens.size(); 
    }

    Token previous() {
        return tokens[pos - 1];
    }

    Token expect(TokenType expectedType) {
        Token tok = peek();  // 查看当前 token，不移动指针
        if (tok.type != expectedType) {
            throw std::runtime_error(
                "Syntax（语法分析器） error: expected " + std::to_string((int)(expectedType)) +
                ", got " + std::to_string((int)(tok.type))+" at line "
                +std::to_string(tok.line)+" pos= "+ std::to_string(pos)
            );
        }
        advance();  // 指针向下移动，表示 token 已被消费
        return tok;
    }

    void skipNewlines() {
        while (match(TokenType::NEWLINE)) {
            advance();
        }
    }


    std::vector<Param> parseParamList();

public:
	Parser(const std::vector<Token>& tokens) : 
		tokens(tokens), pos(0) {}

    Statement* parseStatement();

    FunctionDef* parseFunction();

    Statement* parseInput();

    Statement* parseOutput();

    Statement* parseIf();

    Statement* parseReturn();

    Statement* parseConstDecl();

    ForStmt* parseForStmt();

    WhileStmt* parseWhileStmt();

    Statement* parseExprStatement();
    ExprNode* parseExpression();// 入口，解析最低优先级
    ExprNode* parseAssignment();
    ExprNode* parseLogicalOr();
    ExprNode* parseLogicalAnd();
    ExprNode* parseEquality();
    ExprNode* parseRelational();
    ExprNode* parseAdditive();
    ExprNode* parseMultiplicative();
    ExprNode* parsePrimary();           // 常量、变量、括号、函数调用


    std::vector<Statement*> parseBlock();
};