#include <zmq.hpp>
#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <memory>
#include <cstdlib>
#include <cxxopts.hpp>

// Simple logging setup
class Logger {
public:
    Logger(bool enable_terminal) : terminal(enable_terminal) {
        logfile.open("/var/log/regmsg.log", std::ios::app);
        if (!logfile.is_open()) {
            std::cerr << "Cannot open /var/log/regmsg.log\n";
            std::exit(1);
        }
    }

    void log(const std::string &msg) {
        logfile << msg << std::endl;
        if (terminal) {
            std::cout << msg << std::endl;
        }
    }

private:
    bool terminal;
    std::ofstream logfile;
};

// Command enum equivalent
enum class CommandType {
    ListModes, ListOutputs, CurrentMode, CurrentOutput, CurrentResolution,
    CurrentRotation, CurrentRefresh, CurrentBackend,
    SetMode, SetOutput, SetRotation,
    GetScreenshot, MapTouchScreen, MinToMaxResolution
};

// CLI arguments
struct Cli {
    std::optional<std::string> screen;
    bool log_terminal = false;
    CommandType command;
    std::vector<std::string> args;
    std::string extra_arg; // For commands like SetMode / SetOutput
};

// Build command string to send via ZeroMQ
std::string build_command(const Cli &cli) {
    std::string msg;

    switch (cli.command) {
        case CommandType::ListModes: msg = "listModes"; break;
        case CommandType::ListOutputs: msg = "listOutputs"; break;
        case CommandType::CurrentMode: msg = "currentMode"; break;
        case CommandType::CurrentOutput: msg = "currentOutput"; break;
        case CommandType::CurrentResolution: msg = "currentResolution"; break;
        case CommandType::CurrentRotation: msg = "currentRotation"; break;
        case CommandType::CurrentRefresh: msg = "currentRefresh"; break;
        case CommandType::CurrentBackend: msg = "currentBackend"; break;
        case CommandType::SetMode: msg = "setMode " + cli.extra_arg; break;
        case CommandType::SetOutput: msg = "setOutput " + cli.extra_arg; break;
        case CommandType::SetRotation: msg = "setRotation " + cli.extra_arg; break;
        case CommandType::GetScreenshot: msg = "getScreenshot"; break;
        case CommandType::MapTouchScreen: msg = "mapTouchScreen"; break;
        case CommandType::MinToMaxResolution: msg = "minToMaxResolution"; break;
    }

    if (cli.screen.has_value()) {
        msg += " --screen " + cli.screen.value();
    }

    for (const auto &a : cli.args) {
        msg += " " + a;
    }

    return msg;
}

// Parse CLI arguments using cxxopts
Cli parse_cli(int argc, char **argv) {
    cxxopts::Options options("regmsg-cli", "CLI for regmsg daemon");
    options.add_options()
        ("s,screen", "Target screen", cxxopts::value<std::string>())
        ("l,log", "Enable terminal logging", cxxopts::value<bool>()->default_value("false"))
        ("command", "Subcommand", cxxopts::value<std::string>())
        ("args", "Extra args", cxxopts::value<std::vector<std::string>>())
        ("h,help", "Print usage");

    options.parse_positional({"command", "args"});
    auto result = options.parse(argc, argv);

    if (result.count("help") || !result.count("command")) {
        std::cout << options.help() << std::endl;
        std::exit(0);
    }

    Cli cli;
    if (result.count("screen")) cli.screen = result["screen"].as<std::string>();
    cli.log_terminal = result["log"].as<bool>();

    std::string cmd_str = result["command"].as<std::string>();
    if (cmd_str == "listModes") cli.command = CommandType::ListModes;
    else if (cmd_str == "listOutputs") cli.command = CommandType::ListOutputs;
    else if (cmd_str == "currentMode") cli.command = CommandType::CurrentMode;
    else if (cmd_str == "currentOutput") cli.command = CommandType::CurrentOutput;
    else if (cmd_str == "currentResolution") cli.command = CommandType::CurrentResolution;
    else if (cmd_str == "currentRotation") cli.command = CommandType::CurrentRotation;
    else if (cmd_str == "currentRefresh") cli.command = CommandType::CurrentRefresh;
    else if (cmd_str == "currentBackend") cli.command = CommandType::CurrentBackend;
    else if (cmd_str == "setMode") { cli.command = CommandType::SetMode; cli.extra_arg = result["args"].as<std::vector<std::string>>().at(0); }
    else if (cmd_str == "setOutput") { cli.command = CommandType::SetOutput; cli.extra_arg = result["args"].as<std::vector<std::string>>().at(0); }
    else if (cmd_str == "setRotation") { cli.command = CommandType::SetRotation; cli.extra_arg = result["args"].as<std::vector<std::string>>().at(0); }
    else if (cmd_str == "getScreenshot") cli.command = CommandType::GetScreenshot;
    else if (cmd_str == "mapTouchScreen") cli.command = CommandType::MapTouchScreen;
    else if (cmd_str == "minToMaxResolution") cli.command = CommandType::MinToMaxResolution;
    else { std::cerr << "Unknown command\n"; std::exit(1); }

    if (result.count("args")) cli.args = result["args"].as<std::vector<std::string>>();

    return cli;
}

int main(int argc, char **argv) {
    Cli cli = parse_cli(argc, argv);

    Logger logger(cli.log_terminal);
    logger.log("Starting regmsg-cli");

    try {
        zmq::context_t context(1);
        zmq::socket_t socket(context, zmq::socket_type::req);
        socket.connect("ipc:///var/run/regmsgd.sock");

        std::string cmd = build_command(cli);
        logger.log("Sending command: " + cmd);

        zmq::message_t request(cmd.size());
        memcpy(request.data(), cmd.c_str(), cmd.size());
        socket.send(request, zmq::send_flags::none);

        zmq::message_t reply;
        socket.recv(reply, zmq::recv_flags::none);

        std::string reply_str(static_cast<char*>(reply.data()), reply.size());
        std::cout << reply_str << std::endl;

    } catch (const zmq::error_t &e) {
        std::cerr << "ZeroMQ error: " << e.what() << std::endl;
        return 1;
    }

    return 0;
}
