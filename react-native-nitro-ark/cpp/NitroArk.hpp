#pragma once

#include "HybridNitroArkSpec.hpp"

namespace margelo::nitro::nitroark
{
    class NitroArk : public HybridNitroArkSpec
    {

    public:
        NitroArk() : HybridObject(TAG) {}

    public:
        double multiply(double a, double b) override;
    };
}
