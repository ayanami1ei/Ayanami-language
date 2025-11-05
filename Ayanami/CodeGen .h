#pragma once
#pragma once
#include <fstream>
#include <string>
#include <vector>
#include <unordered_map>
#include <iostream>
#include "IR.h" // 你之前定义的 IRProgram 和 IRInstruction

class CodeGen {
public:
    void generateAndCompile(IRProgram& ir, const std::string& filename, const std::string& outputExe) {
        out.open(filename);
        if (!out.is_open()) {
            throw std::runtime_error("Cannot open output file");
        }

        out << "#include <iostream>\n";
        out << "using namespace std;\n\n";

        // 记录函数返回类型（默认 double）
        currentFunc = "";
        tempVars.clear();

        for (const auto& instr : ir.getInstructions()) {
            genInstruction(instr);
        }

        out.close();

        std::string cmd = "g++ " + filename + " -o " + outputExe;
        int ret = system(cmd.c_str());
        if (ret != 0) {
            throw std::runtime_error("Compilation failed");
        }

        std::cout << "Compilation succeeded: " << outputExe << "\n";
    }

private:
    std::ofstream out;
    std::string currentFunc;
    std::unordered_map<std::string, bool> tempVars; // 临时变量声明标记

    void declareVar(const std::string& name,const TokenType type) {
        if (tempVars.find(name) == tempVars.end()) {
            if (type == TokenType::FLOAT)
                out << "double ";
            else if (type == TokenType::INT)
                out << "int ";
            else if (type == TokenType::BOOL)
                out << "bool ";
            else if (type == TokenType::CHAR)
                out << "char ";
            else if (type == TokenType::ARR_INT)
                out << "int* ";
            else if (type == TokenType::ARR_FLOAT)
                out << "float* ";
            else if (type == TokenType::ARR_BOOL)
                out << "bool* ";
            else if (type == TokenType::ARR_CHAR)
                out << "char* ";
            out << name << ";\n";
            tempVars[name] = true;
        }
    }

    void genInstruction(const IRInstruction& instr) {
        switch (instr.type) {
        case IRType::FUNC_BEGIN:
            currentFunc = instr.result;
            tempVars.clear();
            if (instr.result == "main")
                out << "int ";
            else if (instr.resType == TokenType::FLOAT)
                out << "double ";
            else if (instr.resType == TokenType::INT)
                out << "int ";
            else if (instr.resType == TokenType::BOOL)
                out << "bool ";
            else if (instr.resType == TokenType::CHAR)
                out << "char ";
            else if (instr.resType == TokenType::ARR_INT)
                out << "int* ";
            else if (instr.resType == TokenType::ARR_FLOAT)
                out << "float* ";
            else if (instr.resType == TokenType::ARR_BOOL)
                out << "bool* ";
            else if (instr.resType == TokenType::ARR_CHAR)
                out << "char* ";
            else
                out << "void ";
            out << currentFunc << "(";

            for (int i = 0; i < instr.params.size(); i++) {
                std::string s = instr.params[i];
                std::string from = " Ref";
                std::string to = "&";

                size_t pos = 0;
                while ((pos = s.find(from, pos)) != std::string::npos) {
                    s.replace(pos, from.length(), to);
                    pos += to.length(); // 移动到替换后的字符串末尾，避免死循环
                }

                if (s.substr(0,3)=="ARR") {
                    s = s.substr(4, s.size());
                    for (int k = 0; k < s.size(); k++) {
                        if (s[k] == ' ') {
                            //i--;
                            s.insert(k, "*");
                            break;
                        }
                    }
                }

                out << s;
                if (i != instr.params.size() - 1)
                    out << ",";

                pos = s.rfind(' ', pos);
                s = s.substr(pos + 1, 10000);
                tempVars[s] = true;
            }
            out << ") {\n";
            break;

        case IRType::FUNC_END:
            out << "}\n\n";
            currentFunc = "";
            break;

        case IRType::ADD:
        case IRType::SUB:
        case IRType::MUL:
        case IRType::DIV:
        case IRType::LESS:
        case IRType::GREATER:
        case IRType::EQUAL_EQUAL:
        case IRType::AND:
        case IRType::OR:
            declareVar(instr.result,instr.resType);
            out << instr.result << " = " << instr.op1;
            if (instr.type == IRType::ADD) out << " + ";
            if (instr.type == IRType::SUB) out << " - ";
            if (instr.type == IRType::MUL) out << " * ";
            if (instr.type == IRType::DIV) out << " / ";
            if (instr.type == IRType::LESS) out << " < ";
            if (instr.type == IRType::GREATER) out << " > ";
            if (instr.type == IRType::EQUAL_EQUAL) out << " == ";
            if (instr.type == IRType::AND) out << " && ";
            if (instr.type == IRType::OR) out << " || ";
            out << instr.op2 << ";\n";
            break;

        case IRType::ASSIGN:
            declareVar(instr.result,instr.resType);
            out << instr.result << " = " << instr.op1 << ";\n";
            break;

        case IRType::RETURN:
            out << "return " << instr.op1 << ";\n";
            break;

        case IRType::PARAM:
            // 这里只做函数调用时参数收集
            // 具体生成 call 时处理
            break;

        case IRType::CALL:
            if (instr.resType==TokenType::VOID)
                out << instr.op1 << "(";
            else {
                declareVar(instr.result, instr.resType);
                out << instr.result << " = " << instr.op1 << "(";
            }
            for (int i = 0; i < instr.params.size(); i++) {
                out << instr.params[i];
                if (i != instr.params.size() - 1)
                    out << ",";
            }
            out << ");\n";
            break;
        case IRType::ALLOC_ARR:
            out << instr.op1.substr(4, 10) << "* " << instr.result << " = new "
                << instr.op1.substr(4, 10) << "[" << instr.op2 << "];\n";
            break;
        case IRType::STORE_ARR:
            out << instr.op1 << "[" << instr.op2 << "] = " << instr.result << ";\n";
            break;
#if 0
        case IRType::LOAD_ARR:
            declareVar(instr.result, instr.resType);
            out << instr.result <<" = " << instr.op1 << "[" << instr.op2 << "]" << ";\n";
            break;
#endif
        case IRType::LABEL:
            out << instr.result << ":\n";
            break;

        case IRType::GOTO:
            out << "goto " << instr.result << ";\n";
            break;

        case IRType::IF_TRUE_GOTO:
            out << "if(" << instr.op1 << ") goto " << instr.result << ";\n";
            break;

        case IRType::IF_FALSE_GOTO:
            out << "if(!(" << instr.op1 << ")) goto " << instr.result << ";\n";
            break;
        case IRType::INPUT:
            out << "std::cin>>" << instr.result << ";\n";
            break;
        case IRType::OUTPUT:
            out << "std::cout<<" << instr.result << ";\n";
            break;
        default:
            std::cerr << "Unsupported IRType\n";
        }
    }
};
