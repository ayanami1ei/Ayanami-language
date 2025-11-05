#include <vector>
#ifndef UTIL_H
#define UTIL_H
#include"util.h"
#endif
#include "utf8.h"

#ifndef ASTNODE_H
#define ASTNODE_H
#endif

enum class SymbolType { VAR, CONST, FUNC };

inline std::string valueTypeToString(TokenType a) {
	switch (a) {
	case TokenType::INT:
		return "int";
	case TokenType::FLOAT:
		return "float";
	case TokenType::CHAR:
		return "char";
	case TokenType::BOOL:
		return "bool";
	case TokenType::ARR_INT:
		return "ARR_int";
	case TokenType::ARR_FLOAT:
		return "ARR_float";
	case TokenType::ARR_CHAR:
		return "ARR_char";
	case TokenType::ARR_BOOL:
		return "ARR_bool";
	case TokenType::ARR_ARR:
		return "ARR_bool";
	case TokenType::FN:
		return "fn";
	case TokenType::VOID:
		return "void";
	}
	return "unknown";
}

inline TokenType stringTovalueType(std::string a) {
	if (a == "int")
		return TokenType::INT;
	else if (a == "double")
		return TokenType::FLOAT;
	else if(a == "char")
		return TokenType::CHAR;
	else if(a == "bool")
		return TokenType::BOOL;
	else if(a == "ARR_int")
		return TokenType::ARR_INT;
	else if (a == "ARR_double")
		return TokenType::ARR_FLOAT;
	else if (a == "ARR_char")
		return TokenType::ARR_CHAR;
	else if (a == "ARR_bool")
		return TokenType::ARR_BOOL;
	else if (a == "void")
		return TokenType::VOID;
}

#if 0
struct Symbol {
	std::string name;
	SymbolType symType;  // VAR / CONST / FUNC
	TokenType valueType; // INT / FLOAT / ...
	bool isRef;          // 是否为引用类型
	bool isInitialized;
	int scopeDepth;      // 当前作用域层级
};
#endif

/*
* 含义：所有 AST 节点的父类
* 作用：方便多态操作，Parser / 语义分析器可以用 ASTNode * 指针
* 处理所有节点
*/
class ASTNode {
public:
	virtual ~ASTNode() = default;
};

/*
* 表达式节点基类
* 含义：表示所有可以产生值的语法元素
* 作用：统一表达式接口，方便类型推断、运算符解析
*/
class ExprNode : public ASTNode {
public:
	std::vector<uint32_t> name;
	ExprNode() {}
	ExprNode(const std::vector<uint32_t>& name) :name(name) {}
	virtual bool isAssignable() const { return false; }
};

class CallExpr : public ExprNode {
public:
	std::vector<uint32_t> callee;  // 函数名
	std::vector<ExprNode*> args;   // 参数表达式列表
	CallExpr(const std::vector<uint32_t>& callee, const std::vector<ExprNode*>& args)
		: callee(callee), args(args) {

	}
};


/*
* 数字字面量
* 含义：表示整数或浮点数字面量
* 作用：
*	类型推断阶段：推断为 int 或 float
*	代码生成阶段：生成常量加载指令
*/
class NumberExpr :public ExprNode {
public:
	double value;

	NumberExpr(const std::vector<uint32_t>& v):
		value(std::stod(uint32tsToString(v))) {}

	NumberExpr(double val) :value(val) {}
};

/*
* 字符字面量
* 含义：表示字符字面量
* 作用：
*	类型推断阶段：推断为 char
*	代码生成阶段：生成常量加载指令
*/
class CharExpr :public ExprNode {
public:
	std::string value;

	CharExpr(const std::vector<uint32_t>& v) :
		value(uint32tsToString(v)) {
	}

	CharExpr(std::string& val) :value(val) {}
};

/*
* 布尔字面量
* 含义：表示布尔字面量
* 作用：
*	类型推断阶段：推断为 bool
*	代码生成阶段：生成常量加载指令
*/
class BoolExpr :public ExprNode {
public:
	bool value;

	BoolExpr(const std::vector<uint32_t>& v){
		std::string s = uint32tsToString(v);
		if (s == "true")
			value = true;
		else
			value = false;
	}

	BoolExpr(TokenType t) {
		if (t == TokenType::TRUE)
			value = true;
		else
			value = false;
	}

	BoolExpr(bool val) :value(val) {}
};

/*
变量引用
* 含义：表示访问一个变量
* 作用：
*	类型推断：查符号表确定类型
*	代码生成：加载变量值或引用
*/
class VarExpr :public ExprNode {
public:

	VarExpr(const std::vector<uint32_t>& name):
		ExprNode(name){ }
	virtual bool isAssignable() const override { return true; }
};

class ArrayExpr :public ExprNode {
public:
	std::vector<ExprNode*> elem;
	TokenType type;

	ArrayExpr(const std::vector<ExprNode*>& elem, const std::vector<uint32_t>& name) :
		elem(elem), ExprNode(name) {
	}

	ArrayExpr(const std::vector<ExprNode*>& elem) :
		elem(elem){
	}
};

class ArrayElemExpr :public ExprNode {
public:
	ExprNode* index;
	bool isAssignable() const override { return true; }
	ArrayElemExpr(const std::vector<uint32_t>& name,ExprNode* index) :index(index), ExprNode(name) {}
};

/*
* 二元运算表达式
* 含义：表示两个操作数和一个运算符的表达式
* 作用：
*	类型推断：检查左右操作数类型是否支持运算符
*	运算符重载：语义分析阶段绑定具体函数
*	代码生成：生成相应的加/减/乘/除指令
*/
class BinaryExpr :public ExprNode {
public:
	ExprNode* left;
	std::vector<uint32_t> op;
	ExprNode* right;

	BinaryExpr(ExprNode* left,
	const std::vector<uint32_t>& op,
	ExprNode* right):
		left(left),op(op),right(right){ }

	BinaryExpr(ExprNode* left,
		const std::string& op,
		ExprNode* right) :
		left(left), op(stringToUint32ts(op)), right(right) {
	}
};

//------------------------------------------

/*
* 语句基类
* 含义：表示所有语句节点（不产生值，仅执行动作）
* 作用：区分表达式和语句，便于语义分析器和代码生成器处理
*/
class Statement :public ASTNode {};

/* 
* 表达式语句
* 含义：把一个表达式当作语句执行（如函数调用、纯计算）
*/ 
class ExprStmt :public Statement {
public:
	ExprNode* expr;

	ExprStmt(ExprNode* e) : expr(e) {}
};

/*
* 赋值语句
* 含义：变量赋值操作
* 作用：
*	检查变量是否已声明或是否为 const
*	绑定右侧表达式值到左侧变量
*/
class AssignStmt :public Statement {
public:
	std::vector<uint32_t> varName;
	ExprNode* value;
#if 0
	Symbol* symbol = nullptr; // 在语义分析阶段绑定符号表
#endif
	bool isConst = false;

	AssignStmt() {}

	AssignStmt(const std::vector<uint32_t>& name, ExprNode* value, bool isConst) :
		varName(name), value(value), isConst(isConst) {
	}

	AssignStmt(BinaryExpr* b) {
		varName = b->left->name;
		isConst = false;
	}
};

/*
* 条件语句
* 含义：表示 if 条件块
* 作用：
* 条件求值
* 执行 body 中语句
*/
class IfStmt :public Statement {
public:
	ExprNode* condition;
	std::vector<Statement*> body;

	IfStmt(ExprNode* condition,
		const std::vector<Statement*>& body) :
		condition(condition), body(body) {
	}
};

/*
* 返回语句
* 含义：表示函数返回语句
* 作用：
*	类型推断阶段：确定函数返回类型
*	生成代码阶段：生成 return 指令
*/
class ReturnStmt :public Statement {
public:
	ExprNode* value;

	ReturnStmt(ExprNode* value):value(value){}
};

/*
* 函数定义
* 含义：表示一个函数的定义
* 作用：
*	记录函数名、参数、作用域内语句
*	支持类型推断与参数传递检查
*/
class FunctionDef :public Statement {
public:
	std::vector<uint32_t> name;
	std::vector<Param> params; // (type, name)
	std::vector<Statement*> body;
	TokenType retType;

	FunctionDef(const std::vector<uint32_t>& name,
		const std::vector<Param>& params,
		const std::vector<Statement*>& body) :
		name(name), params(params), body(body) {
	}
};

/*
* for循环
*/
class ForStmt :public Statement {
public:
	ExprNode* param;
	ExprNode* startExpr;  // a
	ExprNode* endExpr;    // b
	ExprNode* stepExpr;   // c，可为 nullptr
	std::vector<Statement*> body;

	ForStmt(ExprNode* param,
		ExprNode* startExpr,  // a
	ExprNode* endExpr,    // b
	ExprNode* stepExpr,   // c，可为 nullptr
		const std::vector<Statement*>& body) :
		param(param), startExpr(startExpr), endExpr(endExpr),
		stepExpr(stepExpr), body(body) {
	}
};

/*
* while循环
*/
class WhileStmt :public Statement {
public:
	ExprNode* condition;
	std::vector<Statement*> body;

	WhileStmt(ExprNode* condition,
		const std::vector<Statement*>& body) :
		condition(condition), body(body) {
	}
};

class InputStmt : public Statement {
public:
	ExprNode* expr;
	InputStmt(ExprNode* e) : expr(e) {}
};

class OutputStmt : public Statement {
public:
	ExprNode* expr;
	OutputStmt(ExprNode* e) : expr(e) {}
};

