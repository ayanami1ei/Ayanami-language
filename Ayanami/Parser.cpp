#include"Parser.h"

//Parser 核心算法
Statement* Parser::parseStatement() {
    if (check(TokenType::END_OF_FILE)) {
        return NULL;
    }
    // 跳过空行（允许连续空行）
    while (match(TokenType::NEWLINE)) 
        continue;

    Token tok = peek();

    Statement* stmt = nullptr;
    if (tok.type == TokenType::FN) {
        stmt = parseFunction();
    }
    else if (tok.type == TokenType::IF) {
        stmt = parseIf();
    }
    else if (tok.type == TokenType::RETURN) {
        stmt = parseReturn();
    }
    else if (tok.type == TokenType::CONST) {
        stmt = parseConstDecl();
    }
    else if (tok.type == TokenType::FOR) {
        stmt = parseForStmt();
    }
    else if (tok.type == TokenType::WHILE) {
        stmt = parseWhileStmt();
    }
    else if (tok.type == TokenType::INPUT) {
        stmt = parseInput();
    }
    else if (tok.type == TokenType::OUTPUT) {
        stmt = parseOutput();
    }
    else {
        stmt = parseExprStatement();
    }

    while (match(TokenType::NEWLINE))
        continue;

    return stmt;
}

ForStmt* Parser::parseForStmt() {
    expect(TokenType::FOR);               // 必须是 for
    auto paramToken = peek();
    ExprNode* param;

    if (paramToken.type == TokenType::IDENTIFIER) {
        param = new VarExpr(paramToken.lexeme);
    }
    else {
        throw std::runtime_error(
            "Syntax（语法分析器） error: wrong type at first pos in for, expect identifier"
        );
    }

#if 0
    else if (paramToken.type == TokenType::INT_LITERAL || 
        paramToken.type == TokenType::FLOAT_LITERAL || 
        paramToken.type == TokenType::CHAR_LITERAL) {

    }

#endif

    advance();

    expect(TokenType::IN);               // 必须是 in

    ExprNode* start, *end, *step=new NumberExpr(1);

    expect(TokenType::LPAREN);               // 必须是 (
    start = parseExpression();
    if (check(TokenType::COMMA)) {
        advance();
        end = parseExpression();
        if (check(TokenType::COMMA)) {
            advance();
            step = parseExpression();
        }
    }
    else {
        end = start;
        start = new NumberExpr(0);
    }
    consume(TokenType::RPAREN, "Expected ')'");// 必须是 )

    auto body = parseBlock();           // 解析函数体

    // 跳过前导换行符（允许空行）
    while (match(TokenType::NEWLINE)) {
        advance(); // 跳过空行
    }

    return new ForStmt{ param, start, end, step, body };
}

WhileStmt* Parser::parseWhileStmt() {
    expect(TokenType::WHILE);               // 必须是 while

    ExprNode* param = parseExpression();

    auto body = parseBlock();           // 解析函数体

    // 跳过前导换行符（允许空行）
    while (match(TokenType::NEWLINE)) {
        advance(); // 跳过空行
    }

    return new WhileStmt{ param, body };
}

FunctionDef* Parser::parseFunction() {
    expect(TokenType::FN);               // 必须是 fn
    std::vector<uint32_t> name = expect(TokenType::IDENTIFIER).lexeme;  // 必须是标识符
    expect(TokenType::LPAREN);           // 必须是 (
    auto params = parseParamList();    // 解析参数列表
    expect(TokenType::RPAREN);           // 必须是 )
    auto body = parseBlock();           // 解析函数体

    // 跳过前导换行符（允许空行）
    while (match(TokenType::NEWLINE)) {
        advance(); // 跳过空行
    }

    return new FunctionDef{ name, params, body };
}

Statement* Parser::parseIf() {
    consume(TokenType::IF, "expect 'if'");
    ExprNode* condition = parseExpression();

    consume(TokenType::LBRACE, "expect '{' after if condition");
    std::vector<Statement*> body;
    while (!check(TokenType::RBRACE) && !isAtEnd()) {
        body.push_back(parseStatement());
    }
    consume(TokenType::RBRACE, "expect '}' after if block");

    return new IfStmt(condition, body);
}

Statement* Parser::parseReturn() {
    advance(); // consume 'return'
    ExprNode* value = nullptr;
    if (!check(TokenType::NEWLINE) && !check(TokenType::END_OF_FILE)) {
        value = parseExpression();
    }

    if (match(TokenType::NEWLINE) || check(TokenType::END_OF_FILE)) {
        return new ReturnStmt(value);
    }else{
        throw std::runtime_error(
            "Expected newline after statement at pos " +
            std::to_string(pos)
        );
    }
}

std::vector<Param> Parser::parseParamList() {
    std::vector<Param> params;

    // 如果遇到 ')'，参数列表为空
    if (peek().type == TokenType::RPAREN) {
        return params;
    }

    while (true) {
        Param p;

        // 是否有 ref 修饰
        if (match(TokenType::REF)) {
            p.isRef = true;
        }
        else {
            p.isRef = false;
        }

        // 类型
        Token typeToken = expect(peek().type); // int / float / char / bool
        p.type = typeToken.type;

        // 参数名
        Token nameToken = expect(TokenType::IDENTIFIER);
        p.name = nameToken.lexeme;

        if (check(TokenType::LSQUARE)) {
            advance();
            expect(TokenType::RSQUARE);
            p.type = (TokenType)((int)p.type - (int)TokenType::INT + (int)TokenType::ARR_INT);
        }

        params.push_back(p);

        // 如果下一个 token 是 ','，继续解析下一个参数
        if (match(TokenType::COMMA)) {
            continue;
        }
        else {
            break;
        }
    }

    return params;
}

Statement* Parser::parseConstDecl() {
    advance(); // consume 'const'

    Token name = expect(TokenType::IDENTIFIER);

    if (!match(TokenType::EQUAL)) {
        throw std::runtime_error("Expected '=' after const variable name");
    }
    //advance(); // consume '='

    ExprNode* value = parseExpression();

    // 构造一个 AssignStmt 节点
    auto node = new AssignStmt();
    node->varName = name.lexeme;
    node->value = value;
    node->isConst = true; //标记常量

    return node;
}

Statement* Parser::parseExprStatement() {
    ExprNode* expr = parseExpression();
    return new ExprStmt(expr);
}

ExprNode* Parser::parseExpression() {
    return parseAssignment();
}

ExprNode* Parser::parseAssignment() {
    ExprNode* left = parseLogicalOr();

    // 只有左边是变量或者属性等可赋值对象时才允许 =
    if (match(TokenType::EQUAL)) {
        if (!left->isAssignable()) {
            throw std::runtime_error("Invalid assignment target");
        }
        ExprNode* right = parseAssignment();
        return new BinaryExpr(left, "=", right);
    }

    return left;
}

ExprNode* Parser::parseLogicalOr() {
    ExprNode* left = parseLogicalAnd();
    while (match(TokenType::OR)) { // ||
        ExprNode* right = parseLogicalAnd();
        left = new BinaryExpr(left, stringToUint32ts("||"), right);
    }
    return left;
}

ExprNode* Parser::parseLogicalAnd() {
    ExprNode* left = parseEquality();
    while (match(TokenType::AND)) { // &&
        ExprNode* right = parseEquality();
        left = new BinaryExpr(left, stringToUint32ts("&&"), right);
    }
    return left;
}

ExprNode* Parser::parseEquality() {
    ExprNode* left = parseRelational();
    while (true) {
        if (match(TokenType::EQUAL_EQUAL)) { // ==
            ExprNode* right = parseRelational();
            left = new BinaryExpr(left,"==", right);
        }
        else if (match(TokenType::NOT_EQUAL)) { // !=
            ExprNode* right = parseRelational();
            left = new BinaryExpr(left, "!=", right);
        }
        else break;
    }
    return left;
}

ExprNode* Parser::parseRelational() {
    ExprNode* left = parseAdditive();
    while (true) {
        if (match(TokenType::LESS)) {
            ExprNode* right = parseAdditive();
            left = new BinaryExpr(left, "<", right);
        }
        else if (match(TokenType::GREATER)) {
            ExprNode* right = parseAdditive();
            left = new BinaryExpr(left, ">", right);
        }
        else break;
    }
    return left;
}

ExprNode* Parser::parseAdditive() {
    ExprNode* left = parseMultiplicative();
    while (true) {
        if (match(TokenType::ADD)) {
            ExprNode* right = parseMultiplicative();
            left = new BinaryExpr(left, "+", right);
        }
        else if (match(TokenType::SUB)) {
            ExprNode* right = parseMultiplicative();
            left = new BinaryExpr(left, "-", right);
        }
        else break;
    }
    return left;
}

ExprNode* Parser::parseMultiplicative() {
    ExprNode* left = parsePrimary();
    while (match(TokenType::MUL) || match(TokenType::DEV)) {
        Token op = previous();
        ExprNode* right = parseMultiplicative(); // 递归，保证右结合
        left = new BinaryExpr(left, op.lexeme, right);
    }
    return left;
}

ExprNode* Parser::parsePrimary() {
    Token cur = peek();
    //std::cout << "parsePrimary(): token=" << cur
    //    << " value=" << uint32tsToString(peek().lexeme) <<" pos= "<<pos << std::endl;

    if (match(TokenType::INT_LITERAL) || match(TokenType::FLOAT_LITERAL)) {
        return new NumberExpr(previous().lexeme);
        
    }
    else if (match(TokenType::CHAR_LITERAL)) {
        return new CharExpr(previous().lexeme);
    }
    if (match(TokenType::IDENTIFIER))  {
        std::vector<uint32_t> name = previous().lexeme;
        if (match(TokenType::LPAREN)) {
            std::vector<ExprNode*> args;
            if (!check(TokenType::RPAREN)) {
                do {
                    args.push_back(parseExpression());
                } while (match(TokenType::COMMA));
            }
            consume(TokenType::RPAREN, "Expected ')'");
            return new CallExpr(name, args);
        }
        if (match(TokenType::LSQUARE)) {
            ExprNode* index = parseExpression();
            consume(TokenType::RSQUARE, "Expected ']' after index");
            return new ArrayElemExpr(name, index);
        }
        return new VarExpr(name);
    }
    if (match(TokenType::TRUE))  
        return new BoolExpr(true);
    if (match(TokenType::FALSE))
        return new BoolExpr(false);
    if (match(TokenType::LPAREN)) {
        ExprNode* expr = parseExpression();
        consume(TokenType::RPAREN, "Expected ')'");
        return expr;
    }
    if (match(TokenType::LSQUARE)) {
        std::vector<ExprNode*> args;
        if (!check(TokenType::RSQUARE)) {
            do {
                args.push_back(parseExpression());
            } while (match(TokenType::COMMA));
        }
        consume(TokenType::RSQUARE, "Expected ']'");
        return new ArrayExpr(args);
    }

   // std::cout << "parsePrimary(): token=" << cur
   //     << " value=" << uint32tsToString(peek().lexeme) << " pos= " << pos << std::endl;
    throw std::runtime_error(
        "Unexpected token, pos "+std::to_string(pos)+"\n"
    );
    return nullptr;
}

std::vector<Statement*> Parser::parseBlock() {
    expect(TokenType::LBRACE);

    std::vector<Statement*> stmts;
    while (!check(TokenType::RBRACE) && !check(TokenType::END_OF_FILE)) {
        stmts.push_back(parseStatement());
    }

    expect(TokenType::RBRACE); // 明确 consume }

    return stmts;
}

Statement* Parser::parseInput() {
    advance();

    expect(TokenType::LPAREN);

    ExprNode* str = parseExpression();

    expect(TokenType::RPAREN);

    return new InputStmt(str);
}

Statement* Parser::parseOutput() {
    advance();

    expect(TokenType::LPAREN);

    ExprNode* str = parseExpression();

    expect(TokenType::RPAREN);

    return new OutputStmt(str);
}