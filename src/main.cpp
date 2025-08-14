#include <cstdlib>
#include <print>

#ifndef CPPHTTPLIB_OPENSSL_SUPPORT
#define CPPHTTPLIB_OPENSSL_SUPPORT
#endif
#include "httplib.h"

int main(void)
{
    httplib::Client cli("http://yhirose.github.io");
    cli.set_max_timeout(10);
    auto res = cli.Get("/hi");
    if (res)
        std::println("Status: {}\nBody:\n{}", res->status, res->body);
    else
        std::cout << res.error() << "\n";

    return EXIT_SUCCESS;
}
