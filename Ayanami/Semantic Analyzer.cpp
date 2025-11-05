#include"Semantic Analyzer.h"
#include <unordered_set>

std::ostream& operator<<(std::ostream& out, Symbol& a) {
    out << "name: " << a.name << std::endl
        << "valueType: " << valueTypeToString(a.valueType) << std::endl
        << "isConst: " << a.isConst << std::endl
        << "isRef: " << a.isRef << std::endl
        << "isFunc: " << a.isFunction << std::endl << std::endl;

    return out;
}

TokenType makeArrayType(TokenType t) {
    switch (t) {
    case TokenType::INT: return TokenType::ARR_INT;
    case TokenType::FLOAT: return TokenType::ARR_FLOAT;
    case TokenType::CHAR: return TokenType::ARR_CHAR;
    case TokenType::BOOL: return TokenType::ARR_BOOL;
    default: return TokenType::UNKNOWN;
    }
}

void SymbolTable::declare(const std::string& name, const Symbol& sym) {
    if (table.find(name) != table.end()) {
        throw std::runtime_error("Symbol '" + name + "' already declared in this scope");
    }
    table[name] = sym;
}

Symbol* SymbolTable::lookup(const std::string& name) {
    auto it = table.find(name);
    if (it != table.end()) return &it->second;
    if (parent) return parent->lookup(name);
    return nullptr;
}

bool SymbolTable::existsInCurrentScope(const std::string& name) {
    return table.find(name) != table.end();
}

SemanticAnalyzer::SemanticAnalyzer() {
    current = new SymbolTable(nullptr); // 全局作用域
    functionReturnStack.clear();
}

SemanticAnalyzer::~SemanticAnalyzer() {
}

void SemanticAnalyzer::analyze(Statement* stmt) {
    visitStatement(stmt);
}

void SemanticAnalyzer::enterScope() {
    SymbolTable* child = new SymbolTable(current);
    current = child;
}

void SemanticAnalyzer::exitScope() {
    SymbolTable* parent = current->parent;
    historySymTable.push_back(current);
    //delete current;
    current = parent;
}

void SemanticAnalyzer::visitStatement(Statement* s) {
    if (auto es = dynamic_cast<ExprStmt*>(s)) { visitExpr(es); return; }
    if (auto as = dynamic_cast<AssignStmt*>(s)) { visitAssign(as); return; }
    if (auto fd = dynamic_cast<FunctionDef*>(s)) { visitFunctionDef(fd); return; }
    if (auto rs = dynamic_cast<ReturnStmt*>(s)) { visitReturn(rs); return; }
    if (auto is = dynamic_cast<IfStmt*>(s)) { visitIf(is); return; }
    if (auto fs = dynamic_cast<ForStmt*>(s)) { visitFor(fs); return; }
    if (auto ws = dynamic_cast<WhileStmt*>(s)) { visitWhile(ws); return; }

    // 其他 statement 类型（If/While/ExprStmt等）可以在此扩展
    // 若是复合语句/块，可进行 enterScope/exitScope 包裹
}

void SemanticAnalyzer::visitExpr(ExprStmt* node) {
    if (auto be = dynamic_cast<BinaryExpr*>(node->expr)) {
        std::string name = uint32tsToString(be->left->name);
        Symbol* sym = current->lookup(name);
        TokenType rhsType = inferType(be->right);

        if (auto elem = dynamic_cast<ArrayElemExpr*>(be->left)) {

            if (!sym)
                throw std::runtime_error("Undeclared array variable '" + name + "'");

            // 检查下标类型
            if (inferType(elem->index) != TokenType::INT)
                throw std::runtime_error("Array index must be integer");

            // 获取数组元素类型
            TokenType elemType;
            switch (sym->valueType) {
            case TokenType::ARR_INT:   elemType = TokenType::INT; break;
            case TokenType::ARR_FLOAT: elemType = TokenType::FLOAT; break;
            case TokenType::ARR_CHAR:  elemType = TokenType::CHAR; break;
            case TokenType::ARR_BOOL:  elemType = TokenType::BOOL; break;
            default:
                throw std::runtime_error("Variable '" + name + "' is not an array");
            }

            // 比较右值类型
            if (elemType != rhsType &&
                rhsType != TokenType::UNKNOWN &&
                elemType != TokenType::UNKNOWN)
            {
                throw std::runtime_error("Type mismatch: assigning " +
                    valueTypeToString(rhsType) +
                    " to element of array '" + name +
                    "' of type " + valueTypeToString(elemType));
            }

            // 若数组尚未确定元素类型（如空数组 []），推断之
            if (sym->valueType == TokenType::UNKNOWN) {
                sym->valueType = makeArrayType(rhsType);
            }

            return;
        }


        if (!sym) {
            // 首次出现 => 声明（你的语法规则）
            Symbol s;
            s.name = name;
            s.valueType = rhsType;
            s.isConst = false;
            current->declare(name, s);

            //std::cout << s;
            return;
        }
        else {
            // 已经存在
            if (sym->isConst) {
                throw std::runtime_error("Cannot assign to const variable '" + name + "'");
            }
            // 类型不一致则尽量报错（你若要允许隐式转换，可放宽）
            if (sym->valueType != rhsType && sym->valueType != TokenType::UNKNOWN && rhsType != TokenType::UNKNOWN) {
                throw std::runtime_error("Type mismatch: assigning " + valueTypeToString(rhsType) +
                    " to variable '" + name + "' of type " + valueTypeToString(sym->valueType));
            }
            // 若符号类型未知，则完善类型信息
            if (sym->valueType == TokenType::UNKNOWN) sym->valueType = rhsType;

            //std::cout << (*sym);
        }
    }
    else if (auto call = dynamic_cast<CallExpr*>(node->expr)) {
        // 分析函数调用
        std::string funcName = uint32tsToString(call->callee);
        for (int i = 0; i < call->args.size(); i++) {
            auto t = inferType(call->args[i]);
            funcName += valueTypeToString(t);
        }
        const Symbol* fn = current->lookup(funcName);
        if (!fn) {
            throw std::runtime_error("Undefined function: " + uint32tsToString(call->callee));
        }

        // 参数数量检查
        if (call->args.size() != fn->paramTypes.size()) {
            throw std::runtime_error(
                "Function '" + uint32tsToString(call->callee) + "' expects " +
                std::to_string(fn->paramTypes.size()) + " arguments, but got " +
                std::to_string(call->args.size()));
        }

        // 参数类型检查
        for (size_t i = 0; i < call->args.size(); ++i) {
            TokenType argType = inferType(call->args[i]);
            if (argType != fn->paramTypes[i]) {
                throw std::runtime_error(
                    "Type mismatch in argument " + std::to_string(i + 1) +
                    " of function '" + uint32tsToString(call->callee) + "'");
            }
        }

        // 返回函数的返回值类型
        //return fn->funcReturnType;
    }
}

void SemanticAnalyzer::visitAssign(AssignStmt* node) {
    std::string name = uint32tsToString(node->varName);
    TokenType rhsType = inferType(node->value);

    if (rhsType>=TokenType::ARR_INT) {
        // 空数组的类型可能未知
        if (rhsType == TokenType::UNKNOWN) {
            // 允许先声明，后续赋值再确定类型
            rhsType = TokenType::UNKNOWN;
        }
    }

    Symbol* sym = current->lookup(name);
    if (!sym) {
        // 首次出现 => 声明（你的语法规则）
        Symbol s;
        s.name = name;
        s.valueType = rhsType;
        s.isConst = node->isConst;
        current->declare(name, s);

        //std::cout << s;
        return;
    }
    else {
        // 已经存在
        if (sym->isConst) {
            throw std::runtime_error("Cannot assign to const variable '" + name + "'");
        }
        // 类型不一致则尽量报错（你若要允许隐式转换，可放宽）
        if (sym->valueType != rhsType && sym->valueType != TokenType::UNKNOWN && rhsType != TokenType::UNKNOWN) {
            throw std::runtime_error("Type mismatch: assigning " + valueTypeToString(rhsType) +
                " to variable '" + name + "' of type " + valueTypeToString(sym->valueType));
        }
        // 若符号类型未知，则完善类型信息
        if (sym->valueType == TokenType::UNKNOWN) sym->valueType = rhsType;

        //std::cout << (*sym);
    }
}

void SemanticAnalyzer::visitFunctionDef(FunctionDef* node) {
    std::string fname = uint32tsToString(node->name);
    for (int i = 0; i < node->params.size(); i++) {
        fname += valueTypeToString(node->params[i].type);
    }

    if (current->lookup(fname) && current->existsInCurrentScope(fname)) {
        throw std::runtime_error("Function '" + fname + "' already declared in this scope");
    }

    // 在当前作用域声明函数符号（占位）
    Symbol funcSym;
    funcSym.name = fname;
    funcSym.isFunction = true;
    funcSym.valueType = TokenType::FN;

    // 参数类型/名字写入 funcSym.paramTypes（如果 param.type 已知）
    for (const Param& p : node->params) {
        funcSym.paramTypes.push_back(p.type);
    }
    current->declare(fname, funcSym);

    // 新作用域（函数体）并把参数导入符号表
    enterScope();
    for (const Param& p : node->params) {
        std::string pname = uint32tsToString(p.name);
        Symbol psym;
        psym.name = pname;
        psym.valueType = p.type;
        psym.isRef = p.isRef;
        current->declare(pname, psym);
    }

    // 压入一个 return-type set，用于收集函数体所有 return 的类型
    functionReturnStack.emplace_back();
    auto& retSet = functionReturnStack.back();

    // 遍历函数体语句
    for (Statement* st : node->body)
        visitStatement(st);


    // 检查 return set
    if (retSet.empty()) {
        // 没有 return => void
        // 这里可以设置函数符号的返回类型信息
        Symbol* fs = current->parent->lookup(fname); // 注意：函数符号在父作用域
        if (fs) fs->funcReturnType = TokenType::VOID;

        node->retType = TokenType::VOID;
    }
    else if (retSet.size() == 1) {
        TokenType rt = *retSet.begin();
        Symbol* fs = current->parent->lookup(fname);
        if (fs) fs->funcReturnType = rt;

        node->retType = rt;
    }
    else {
        // 多种返回类型，报错（你的语言规范要求）
        throw std::runtime_error("Function '" + fname + "' has multiple return types");
    }

    // 出作用域
    functionReturnStack.pop_back();
    exitScope();
}

void SemanticAnalyzer::visitReturn(ReturnStmt* node) {
    if (functionReturnStack.empty()) {
        throw std::runtime_error("Return statement not inside a function");
    }
    TokenType t = inferType(node->value);
    functionReturnStack.back().insert(t);
}

void SemanticAnalyzer::visitIf(IfStmt* node) {
    // 条件表达式必须是bool
    TokenType condType = inferType(node->condition);
    if (!(condType == TokenType::BOOL || 
        condType == TokenType::TRUE || 
        condType == TokenType::FALSE))
        throw std::runtime_error("Condition of if must be bool");

    // 新作用域：if 块
    enterScope();
    for (Statement* st : node->body)
        visitStatement(st);
    exitScope();
}

void SemanticAnalyzer::visitFor(ForStmt* node) {
    //  处理循环变量声明
    std::string varName = uint32tsToString(node->param->name);
    Symbol* sym = current->lookup(varName);
    //TokenType rhsType = inferType(node->param);

    if (!sym) {
        // 首次出现 => 声明（你的语法规则）
        Symbol funcSym;
        funcSym.name = varName;
        funcSym.valueType = TokenType::INT;
        funcSym.isConst = false;
        current->declare(varName, funcSym);

        //std::cout << s;
        //return;
    }

    // 检查并推导各个表达式的类型
    TokenType startType = TokenType::INT;
    TokenType endType = TokenType::INT;
    TokenType stepType = TokenType::INT;

    if (node->startExpr)
        startType = inferType(node->startExpr);
    if (node->endExpr)
        endType = inferType(node->endExpr);
    if (node->stepExpr)
        stepType = inferType(node->stepExpr);

    // 检查类型一致性
    if (endType != TokenType::INT || startType != TokenType::INT || stepType != TokenType::INT)
        throw std::runtime_error("For-loop bounds and step must be integers.");

    // 省略参数补全默认值
    if (!node->startExpr)
        node->startExpr = new NumberExpr(0);
    if (!node->stepExpr)
        node->stepExpr = new NumberExpr(1);

    // 分析循环体
    enterScope();

    bool prevInLoop = inLoop;
    inLoop = true;

    for (Statement* st : node->body)
        visitStatement(st);

    inLoop = prevInLoop;
    exitScope();


}

void SemanticAnalyzer::visitWhile(WhileStmt* node) {
    // 检查条件表达式类型
    TokenType condType = inferType(node->condition);

    // 允许自动转为bool的类型
    if (condType != TokenType::BOOL &&
        condType != TokenType::INT &&
        condType != TokenType::FLOAT &&
        condType != TokenType::CHAR) {
        throw std::runtime_error("While condition must be convertible to bool");
    }

    //  建立循环作用域
    enterScope();

    //  标记“当前在循环中”，如果你支持 break/continue，可用于校验
    bool prevInLoop = inLoop;
    inLoop = true;

    // 检查循环体语义
    for (Statement* st : node->body)
        visitStatement(st);

    // 恢复循环状态与作用域
    inLoop = prevInLoop;
    exitScope();
}

void SemanticAnalyzer::visitInput(InputStmt* stmt) {
    TokenType type = inferType(stmt->expr);
    if (type == TokenType::UNKNOWN) {
        throw std::runtime_error("Cannot print expression of unknown type");
    }
}

void SemanticAnalyzer::visitOutput(OutputStmt* stmt) {
    TokenType type = inferType(stmt->expr);
    if (type == TokenType::UNKNOWN) {
        throw std::runtime_error("Cannot print expression of unknown type");
    }
}

TokenType SemanticAnalyzer::inferType(ExprNode* expr) {
    if (!expr) return TokenType::VOID;
    if (auto n = dynamic_cast<NumberExpr*>(expr)) {
        // 判断整数或浮点
        double v = n->value;
        if (std::floor(v) == v) return TokenType::INT;
        return TokenType::FLOAT;
    }
    if (auto n = dynamic_cast<CharExpr*>(expr)) {
        // 判断字符
        std::string v = n->value;
        return TokenType::CHAR;
    }
    if (auto n = dynamic_cast<BoolExpr*>(expr)) {
        // 判断布尔
        bool v = n->value;
        return TokenType::BOOL;
    }
    if (auto v = dynamic_cast<VarExpr*>(expr)) {
        std::string name = uint32tsToString(v->name);
        Symbol* sym = current->lookup(name);
        if (!sym) {
            // 变量未声明：按你的规则，第一次出现会在 Assign 时声明，这里作为引用报错
            throw std::runtime_error("Use of undeclared variable '" + name + "'");
        }
        return sym->valueType;
    }
    if (auto ae = dynamic_cast<ArrayExpr*>(expr)) {
        if (ae->elem.size() == 0) {
            return TokenType::UNKNOWN;
        }
        TokenType type = inferType(ae->elem[0]);
        for (int i = 1; i < ae->elem.size(); i++) {
            if (inferType(ae->elem[i]) != type) {
                throw std::runtime_error("There are more than one type in this array");
            }
        }
        ae->type = TokenType((int)type - (int)TokenType::INT + (int)TokenType::ARR_INT);
        return ae->type;
    }
    if (auto elem = dynamic_cast<ArrayElemExpr*>(expr)) {
        Symbol* sym = current->lookup(uint32tsToString(elem->name));
        if (!sym)
            throw std::runtime_error("Undeclared array variable '" + uint32tsToString(elem->name) + "'");
        if (inferType(elem->index) != TokenType::INT)
            throw std::runtime_error("Array index must be integer");

        // 返回数组元素类型
        switch (sym->valueType) {
        case TokenType::ARR_INT: return TokenType::INT;
        case TokenType::ARR_FLOAT: return TokenType::FLOAT;
        case TokenType::ARR_CHAR: return TokenType::CHAR;
        case TokenType::ARR_BOOL: return TokenType::BOOL;
        default:
            throw std::runtime_error("Attempting to index non-array variable '" + uint32tsToString(elem->name) + "'");
        }
    }
    if (auto b = dynamic_cast<BinaryExpr*>(expr)) {
        // 简化处理：如果是赋值 "="，我们在 AssignStmt 里处理；如果是加减乘除则根据左右类型推断
        std::string op = uint32tsToString(b->op);
        if (op == "=") {
            // 不应走到这里：赋值用 AssignStmt 表示（parser 应区分）
            return inferType(b->right);
        }
        TokenType L = inferType(b->left);
        TokenType R = inferType(b->right);
        if (op == "+" || op == "-" || op == "*" || op == "/") {
            if (L == TokenType::FLOAT || R == TokenType::FLOAT) return TokenType::FLOAT;
            if (L == TokenType::INT && R == TokenType::INT) return TokenType::INT;
            if (L == TokenType::CHAR || R == TokenType::CHAR) return TokenType::CHAR;
            if (L == TokenType::BOOL || R == TokenType::BOOL) return TokenType::BOOL;
            // 其他组合暂返回 UNKNOWN
            return TokenType::UNKNOWN;
        }
        // 逻辑/比较运算返回 BOOL
        if (op == "==" || op == "!=" || op == "<" || op == ">" ||
            op == "<=" || op == ">=" || op == "&&" || op == "||") {
            return TokenType::BOOL;
        }

        return TokenType::UNKNOWN;
    }
    if (auto call = dynamic_cast<CallExpr*>(expr)) {
        // 分析函数调用
        std::string funcName = uint32tsToString(call->callee);
        for (int i = 0; i < call->args.size(); i++) {
            auto t = inferType(call->args[i]);
            funcName += valueTypeToString(t);
        }
        const Symbol* fn = current->lookup(funcName);
        if (!fn) {
            throw std::runtime_error("Undefined function: " + uint32tsToString(call->callee));
        }
        
        // 参数数量检查
        if (call->args.size() != fn->paramTypes.size()) {
            throw std::runtime_error(
                "Function '" + uint32tsToString(call->callee) + "' expects " +
                std::to_string(fn->paramTypes.size()) + " arguments, but got " +
                std::to_string(call->args.size()));
        }

        // 参数类型检查
        for (size_t i = 0; i < call->args.size(); ++i) {
            TokenType argType = inferType(call->args[i]);
            if (argType != fn->paramTypes[i]) {
                throw std::runtime_error(
                    "Type mismatch in argument " + std::to_string(i + 1) +
                    " of function '" + uint32tsToString(call->callee) + "'");
            }
        }

        // 返回函数的返回值类型
        return fn->funcReturnType;
    }

    // 其他表达式类型（CallExpr, MemberExpr, etc.）需要扩展
    return TokenType::UNKNOWN;
}





