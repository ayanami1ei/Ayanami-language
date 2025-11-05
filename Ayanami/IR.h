#pragma once
#include <string>
#include <vector>
#include <iostream>
#ifndef ASTNODE_H
#include"ASTNode.h"
#define ASTNODE_H
#endif
#ifndef UTIL_H
#include"util.h"
#define UTIL_H
#endif

#ifndef SEM_H
#define SEM_H
#include"Semantic Analyzer.h"
#endif

// 操作符类型
enum class IRType {
    ADD, SUB, MUL, DIV, ASSIGN,LESS,GREATER,EQUAL_EQUAL,AND,OR,
    LABEL,      // 标签（用于跳转）
    GOTO,       // 无条件跳转
    IF_TRUE_GOTO, // 条件跳转（布尔为真）
    IF_FALSE_GOTO,// 条件跳转（布尔为假）
    RETURN,     // 返回
    CALL,       // 函数调用
    PARAM,      // 调用参数
    FUNC_BEGIN, // 函数开始
    FUNC_END,    // 函数结束
    ALLOC_ARR,
    STORE_ARR,
    LOAD_ARR,
    CONST_BOOL,
    INPUT,
    OUTPUT,
    // 可以扩展更多操作符
};

// 中间表示的基本指令
class IRInstruction {
public:
    IRType type;  // 操作类型（加法、减法等）
    TokenType resType;
    std::string result;  // 结果变量名
    std::string op1;  // 第一个操作数
    std::string op2;  // 第二个操作数（如果有的话）
    std::vector<std::string> params; // 新增：函数参数列表（变量名）

    IRInstruction(IRType type, const std::string& result, const std::string& op1, const std::string& op2 = "", 
        const std::vector<std::string>& params={}, const TokenType resType = TokenType::UNKNOWN)
        : type(type), resType(resType),result(result), op1(op1), op2(op2), params(params){
    }

    void print() const {
        switch (type) {
        case IRType::ADD:       std::cout << result << " = " << op1 << " + " << op2; break;
        case IRType::SUB:       std::cout << result << " = " << op1 << " - " << op2; break;
        case IRType::MUL:       std::cout << result << " = " << op1 << " * " << op2; break;
        case IRType::DIV:       std::cout << result << " = " << op1 << " / " << op2; break;
        case IRType::ASSIGN:    std::cout << result << " = " << op1; break;
        case IRType::LABEL:     std::cout << result << ":"; break;
        case IRType::GOTO:      std::cout << "goto " << result; break;
        case IRType::IF_TRUE_GOTO:  std::cout << "if " << op1 << " goto " << result; break;
        case IRType::IF_FALSE_GOTO: std::cout << "ifFalse " << op1 << " goto " << result; break;
        case IRType::RETURN:    std::cout << "return " << op1; break;
        case IRType::CALL:      std::cout << result << " = " << op1<<" ";
            for (int i = 0; i < params.size(); i++) {
                std::cout << params[i];
                if (i != params.size() - 1)
                    std::cout << ",";
            }
            break;
        case IRType::PARAM:     std::cout << "param " << op1; break;
        case IRType::FUNC_BEGIN:std::cout << "func " << valueTypeToString(resType)<<" " << result << " ";
            for (int i = 0; i < params.size(); i++) {
                std::cout << params[i];
                if (i != params.size() - 1)
                    std::cout << ",";
            }
            std::cout << " begin"; break;
        case IRType::FUNC_END:  std::cout << "func " << result << " end"; break;

            // ==== 新增数组操作 ====
        case IRType::ALLOC_ARR:
            std::cout << op1 <<" " << result << " = alloc " << op1 << " " << op2; // arr = alloc size
            break;
        case IRType::LOAD_ARR:
            std::cout << result << " = " << op1 << "[" << op2 << "]"; // t = arr[i]
            break;
        case IRType::STORE_ARR:
            std::cout << op1 << "[" << op2 << "] = " << result; // arr[i] = t
            break;
        }
        std::cout << std::endl;
    }

};

// IR 程序类
class IRProgram {
private:
    int tempCount = 0;

    SemanticAnalyzer st;

    std::string inferType(CallExpr* node) {
        for (int i = 0; i < st.historySymTable.size(); i++) {
            st.current = st.historySymTable[i];
            try {
                st.inferType(node);

                std::string res;
                for (int i = 0; i < node->args.size(); i++) {
                    auto t = st.inferType(node->args[i]);
                    res += valueTypeToString(t);
                }
                return res;
            }
            catch (const std::exception& ex) {
                continue;
            }
        }

        return "";
    }

    std::string inferType(ExprNode* node) {
        for (int i = 0; i < st.historySymTable.size(); i++) {
            st.current = st.historySymTable[i];
            try {
                return valueTypeToString(st.inferType(node));
            }
            catch (const std::exception& ex) {
                continue;
            }
        }

        return "";
    }

    std::string newTemp() {
        return "t" + std::to_string(++tempCount);
    }

    std::vector<IRInstruction> instructions;  // 存储所有 IR 指令

    // 添加指令
    void addInstruction(const IRInstruction& instr) {
        instructions.push_back(instr);
    }

    void emit(IRType type, const std::string& result, const std::string& op1, const std::string& op2 = "",
        const std::vector<std::string>& params = {}, const TokenType resType = TokenType::UNKNOWN) {
        instructions.emplace_back(type, result, op1, op2, params, resType);
    }


    std::string visitArrayLiteral(ArrayExpr* expr)  {
        size_t n = expr->elem.size();
        std::string arrTemp = newTemp();
        emit(IRType::ALLOC_ARR, arrTemp, valueTypeToString(expr->type), std::to_string(n), {}, expr->type);

        for (size_t i = 0; i < n; ++i) {
            std::string elemTemp = genExpr(expr->elem[i]);
            emit(IRType::STORE_ARR, elemTemp, arrTemp, std::to_string(i), {}, expr->type);
        }

        return arrTemp;
    }

    std::string visitArrayAccess(ArrayElemExpr* expr) {
        std::string arrName = uint32tsToString(expr->name);
        std::string indexTemp = genExpr(expr->index);
        //std::string result = newTemp();

        emit(IRType::LOAD_ARR, arrName, indexTemp, "", {}, stringTovalueType(inferType(expr)));
        return arrName + "[" + indexTemp + "]";
    }

    std::string genExpr(ExprNode* expr) {
        if (auto num = dynamic_cast<NumberExpr*>(expr)) {
            return std::to_string(num->value);
        }
        if (auto c = dynamic_cast<CharExpr*>(expr)) {
            return "\'"+c->value+"\'";
        }
        if (auto var = dynamic_cast<VarExpr*>(expr)) {
            return uint32tsToString(var->name);
        }
        if (auto ae = dynamic_cast<ArrayExpr*>(expr)) {
            return visitArrayLiteral(ae);
        }
        if (auto aee = dynamic_cast<ArrayElemExpr*>(expr)) {
            return visitArrayAccess(aee);
        }
        if (auto bin = dynamic_cast<BinaryExpr*>(expr)) {
            std::string left = genExpr(bin->left);
            std::string right = genExpr(bin->right);
            std::string result = newTemp();

            std::string op = uint32tsToString(bin->op);
            if (op == "+")
                emit(IRType::ADD, result, left, right, {}, stringTovalueType(inferType(bin->right)));
            else if (op == "-")
                emit(IRType::SUB, result, left, right, {}, stringTovalueType(inferType(bin->right)));
            else if (op == "*")
                emit(IRType::MUL, result, left, right, {}, stringTovalueType(inferType(bin->right)));
            else if (op == "/")
                emit(IRType::DIV, result, left, right, {}, stringTovalueType(inferType(bin->right)));
            else if (op == "=") 
                emit(IRType::ASSIGN, left, right, "", {}, stringTovalueType(inferType(bin->right)));
            else if (op == "<") 
                emit(IRType::LESS, result, left, right, {}, TokenType::BOOL);
            else if (op == ">")
                emit(IRType::GREATER, result, left, right, {}, TokenType::BOOL);
            else if (op == "==")
                emit(IRType::EQUAL_EQUAL, result, left, right, {}, TokenType::BOOL);
            else if (op == "&&")
                emit(IRType::AND, result, left, right, {}, TokenType::BOOL);
            else if (op == "||")
                emit(IRType::OR, result, left, right, {}, TokenType::BOOL);

            

            return result;
        }
        if (auto call = dynamic_cast<CallExpr*>(expr)) {
            std::string funcName = uint32tsToString(call->callee);
            funcName += inferType(call);

            std::vector<std::string>paramNames;
            for (auto arg : call->args) {
                std::string val = genExpr(arg);
                paramNames.push_back(val);
            }
            TokenType tret = stringTovalueType(inferType((ExprNode * )call));
            std::string ret = newTemp();
            emit(IRType::CALL, ret, funcName, "", paramNames, tret);
            //addInstruction(IRInstruction(IRType::CALL, ret, fn, std::to_string(call->args.size())));
            return ret;
        }

        return ""; // 其他表达式类型以后再扩展
    }

    int labelCount = 0;

    std::string newLabel(const std::string& prefix = "L") {
        return prefix + std::to_string(++labelCount);
    }


    void visitFunction(FunctionDef* func) {

        std::string funcName = uint32tsToString(func->name);
        for (int i = 0; i < func->params.size(); i++) {
            funcName += valueTypeToString(func->params[i].type);
        }
        std::vector<std::string>paramNames;
        for (auto& i : func->params) {
            std::string s;
            if (i.isRef)
                s = valueTypeToString(i.type) + " Ref " + uint32tsToString(i.name);
            else
                s = valueTypeToString(i.type) + " " + uint32tsToString(i.name);
             paramNames.push_back(s);
        }
        emit(IRType::FUNC_BEGIN, funcName, "", "", paramNames, func->retType);

        // 遍历函数体
        for (auto stmt : func->body)
            visitStatement(stmt);
        addInstruction(IRInstruction(IRType::FUNC_END, funcName, ""));
    }

    void visitReturn(ReturnStmt* stmt) {
        std::string value = stmt->value ? genExpr(stmt->value) : "";
        //addInstruction(IRInstruction(IRType::RETURN, value, ""));
        emit(IRType::RETURN, value, value);
    }

    void visitWhile(WhileStmt* stmt) {
        std::string startLabel = newLabel("while_start");
        std::string endLabel = newLabel("while_end");

        emit(IRType::LABEL, startLabel, "", "");

        if (auto boolExpr = dynamic_cast<BoolExpr*>(stmt->condition)) {
            if (!boolExpr->value) {
                // while(false) 直接跳到 end
                emit(IRType::GOTO, endLabel, "", "");
                emit(IRType::LABEL, endLabel, "", "");
                return;
            }
            // while(true) 不生成条件判断
        }
        else {
            // 普通条件
            std::string condTemp = genExpr(stmt->condition);
            emit(IRType::IF_FALSE_GOTO, endLabel, condTemp, "");
        }

        for (auto& substmt : stmt->body)
            visitStatement(substmt);

        emit(IRType::GOTO, startLabel, "", "");
        emit(IRType::LABEL, endLabel, "", "");
    }

    void visitFor(ForStmt* stmt) {
        std::string iter = uint32tsToString(stmt->param->name);
        std::string start = stmt->startExpr ? genExpr(stmt->startExpr) : "0";
        std::string end = stmt->endExpr ? genExpr(stmt->endExpr) : "0";
        std::string step = stmt->stepExpr ? genExpr(stmt->stepExpr) : "1";

        std::string loopVar = newTemp();
        emit(IRType::ASSIGN, loopVar, start, "", {}, stringTovalueType(inferType(stmt->param)));

        std::string loopStart = newLabel("for_start");
        std::string loopEnd = newLabel("for_end");

        addInstruction(IRInstruction(IRType::LABEL, loopStart, ""));
        std::string condTemp = newTemp();
        //addInstruction(IRInstruction(IRType::SUB, condTemp, end, loopVar));
        emit(IRType::SUB, condTemp, end, loopVar, {}, stringTovalueType(inferType(stmt->endExpr)));
        addInstruction(IRInstruction(IRType::IF_FALSE_GOTO, loopEnd, condTemp));

        //addInstruction(IRInstruction(IRType::ASSIGN, iter, loopVar));
        emit(IRType::ASSIGN, iter, loopVar, "", {}, stringTovalueType(inferType(stmt->param)));
        for (auto& substmt : stmt->body)
            visitStatement(substmt);
        std::string incTemp = newTemp();
        //addInstruction(IRInstruction(IRType::ADD, incTemp, loopVar, step));
        emit(IRType::ADD, incTemp, loopVar, step, {}, stringTovalueType(inferType(stmt->stepExpr)));
        //addInstruction(IRInstruction(IRType::ASSIGN, loopVar, incTemp));
        emit(IRType::ASSIGN, loopVar, incTemp, "", {}, stringTovalueType(inferType(stmt->param)));
        addInstruction(IRInstruction(IRType::GOTO, loopStart, ""));
        addInstruction(IRInstruction(IRType::LABEL, loopEnd, ""));
    }

    void visitIf(IfStmt* stmt) {
        std::string startLabel = newLabel("if_start");
        std::string endLabel = newLabel("if_end");

        emit(IRType::LABEL, startLabel, "", "");

        if (auto boolExpr = dynamic_cast<BoolExpr*>(stmt->condition)) {
            if (!boolExpr->value) {
                // while(false) 直接跳到 end
                emit(IRType::GOTO, endLabel, "", "");
                emit(IRType::LABEL, endLabel, "", "");
                return;
            }
            // while(true) 不生成条件判断
        }
        else {
            // 普通条件
            std::string condTemp = genExpr(stmt->condition);
            emit(IRType::IF_FALSE_GOTO, endLabel, condTemp, "");
        }

        for (auto& substmt : stmt->body)
            visitStatement(substmt);

        //emit(IRType::GOTO, startLabel, "", "");
        emit(IRType::LABEL, endLabel, "", "");
    }
public:

    IRProgram(const SemanticAnalyzer& st) {
        this->st = st;
    }

    void visitStatement(Statement* s) {
        if (auto assign = dynamic_cast<AssignStmt*>(s)) {
            std::string target = uint32tsToString(assign->varName);
            std::string value = genExpr(assign->value);
            addInstruction(IRInstruction(IRType::ASSIGN, target, value));
        }
        else if (auto block = dynamic_cast<IfStmt*>(s)) {
            visitIf(block);
        }
        else if (auto expr = dynamic_cast<ExprStmt*>(s)) {
            genExpr(expr->expr); // 只是计算，不存储结果
        }
        else if (auto rt = dynamic_cast<ReturnStmt*>(s)) {
            visitReturn(rt);
        }
        else if (auto fd = dynamic_cast<FunctionDef*>(s)) {
            visitFunction(fd);
        }
        else if (auto ws = dynamic_cast<WhileStmt*>(s)) {
            visitWhile(ws);
        }
        else if (auto fs = dynamic_cast<ForStmt*>(s)) {
            visitFor(fs);
        }
        else if (auto p = dynamic_cast<InputStmt*>(s)) {
            std::string temp = genExpr(p->expr); // 生成表达式值到临时变量
            emit(IRType::INPUT, temp,"");
        }
        else if (auto p = dynamic_cast<OutputStmt*>(s)) {
            std::string temp = genExpr(p->expr); // 生成表达式值到临时变量
            emit(IRType::OUTPUT, temp, "");
        }
    }

    std::vector<IRInstruction>& getInstructions() {
        return instructions;
    }

    // 打印所有指令
    void print() const {
        for (const auto& instr : instructions) {
            instr.print();
        }
    }
};
