#include <string>
#include <vector>
#include <fstream>
#include"Parser.h"
#include "utf8.h"
#include"Lexer.h"
#include"IR.h"
#include <cstdio>
#include <filesystem>


#ifndef SEM_H
#include"Semantic Analyzer.h"
#define SEM_H
#endif

#include"CodeGen .h"


std::vector<uint32_t> loadSourceFile(const std::string& inputFile) {
    std::ifstream file(inputFile, std::ios::binary);
    std::string bytes((std::istreambuf_iterator<char>(file)),
        std::istreambuf_iterator<char>());

    if (!utf8::is_valid(bytes.begin(), bytes.end()))
        throw std::runtime_error("Invalid UTF-8 in source file");

    std::vector<uint32_t> result;
    auto it = bytes.begin();
    while (it != bytes.end())
        result.push_back(utf8::next(it, bytes.end()));
    return result;
}

int main(int argc, char* argv[]) {
#if not _DEBUG
    if (argc < 2) {
        std::cerr << "用法: ayanami <source.aya> [选项]\n";
        std::cerr << "选项:\n"
            << "  -o <file>     exe文件名\n"
            << "  run           编译并立即执行\n";
        return 1;
    }

    std::string inputFile = argv[1];
    std::string outputFile = inputFile;
    outputFile = outputFile.substr(0, outputFile.size() - 4);
    outputFile += ".cpp";
    bool run = false;

    // 解析命令行参数
    for (int i = 2; i < argc; ++i) {
        std::string arg = argv[i];
        if (arg == "-o" && i + 1 < argc) {
            outputFile = argv[++i];
            outputFile += ".cpp";
        }
        else if (arg == "run") {
            run = true;
        }
    }
#endif
    try {
        std::string inputFile = "test.aya";
        std::string outputFile = "test.cpp";

        std::cerr << "start compiling\n";
        std::vector<uint32_t> src = loadSourceFile(inputFile);

        Lexer lexer(src);

        std::vector<Token> tokens = lexer.tokenize();
        //for (int i = 0; i < tokens.size(); i++) {
        //    std::cout << tokens[i] << " no." << i << std::endl;
        //}

        Parser parser(tokens);
        std::vector<Statement*>res;

       Statement* temp = NULL;
       do {
           temp = parser.parseStatement();
           res.push_back(temp);
       } while (temp != NULL);


        SemanticAnalyzer sema;
        for (auto& i : res) {
            sema.analyze(i);
        }


        IRProgram ir(sema);
        for (auto& i : res) {
            ir.visitStatement(i);
        }
        //ir.print();


        CodeGen cg;
        cg.generateAndCompile(ir, outputFile, outputFile.substr(0, outputFile.size() - 4));

#if not _DEBUG
        std::filesystem::remove(outputFile);
        if (run) {
            outputFile = outputFile.substr(0, outputFile.size() - 4);
            outputFile += ".exe";
            std::cerr << "start " << outputFile << std::endl;
            std::cerr << "\n--------exe output---------\n\n";
            system(outputFile.c_str());
        }
#endif
    }
    catch (const std::exception& ex) {
        std::cout << ex.what();
        return 0;
    }
}