// semantic_analyzer.cpp
// 依赖：你的 AST 定义（上文）以及 util.h 中 utf32ToString 等工具。
// 编译示例：g++ -std=c++17 semantic_analyzer.cpp -o sema

#include <iostream>
#include <string>
#include <vector>
#include <unordered_map>
#include <memory>
#include <stdexcept>
#include <set>
#ifndef ASTNODE_H
#define ASTNODE_H
#include"ASTNode.h"
#endif
#ifndef SEM_H
#define SEM_H
#endif
// 假定 util.h 提供 utf32ToString / stringToUint32ts 等
#ifndef UTIL_H
#define UTIL_H
#include"util.h"
#endif

// ---------- 符号表示 ----------
struct Symbol {
    std::string name;
    TokenType valueType = TokenType::UNKNOWN;
    bool isConst = false;
    bool isRef = false;        // 如果是引用参数等
    bool isFunction = false;
    // 对于函数可以扩展：参数类型列表、返回类型等
    std::vector<TokenType> paramTypes;
    TokenType funcReturnType = TokenType::UNKNOWN;

    friend std::ostream& operator<<(std::ostream& out, Symbol& a);
};

// ---------- 符号表（支持链式作用域） ----------
class SymbolTable {
public:
    SymbolTable(SymbolTable* parent = nullptr) : parent(parent) {}
    ~SymbolTable() {
        //delete parent;
    }

    // 在当前作用域声明（不搜索父作用域）
    void declare(const std::string& name, const Symbol& sym);

    // 搜索到最近的定义（逐级向上）
    Symbol* lookup(const std::string& name);

    // 仅在当前作用域检查是否存在
    bool existsInCurrentScope(const std::string& name);

    SymbolTable* parent;
private:
    std::unordered_map<std::string, Symbol> table;
};


// ---------- 语义分析器 ----------
class SemanticAnalyzer {
public:
    SemanticAnalyzer();

    ~SemanticAnalyzer();

    // 入口：对一条语句进行语义检查
    void analyze(Statement* stmt);

    // 作用域控制
    void enterScope();
    void exitScope();

    // ---------- Expression type inference ----------
    TokenType inferType(ExprNode* expr);

    SymbolTable* current;

    std::vector<SymbolTable*>historySymTable;
private:

    bool inLoop = false;

    // 用于函数内收集 return 类型（支持嵌套函数时使用堆栈）
    std::vector<std::set<TokenType>> functionReturnStack;

    // ---------- Statement visitors ----------
    void visitStatement(Statement* s);

    void visitExpr(ExprStmt* node);

    // AssignStmt: 若变量不存在则声明；若存在做 const 检查与类型匹配
    void visitAssign(AssignStmt* node);

    // FunctionDef: 声明函数签名，检查体内 return 类型一致性
    void visitFunctionDef(FunctionDef* node);

    // ReturnStmt: 只能在函数体内使用；收集当前 return 的类型
    void visitReturn(ReturnStmt* node);

    void visitIf(IfStmt* node);

    void visitFor(ForStmt* node);

    void visitWhile(WhileStmt* node);

    void visitInput(InputStmt* stmt);

    void visitOutput(OutputStmt* stmt);

};

