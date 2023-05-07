#include <stdint.h>
void ___rpsl_abort(uint32_t);
uint32_t ___rpsl_node_call(uint32_t, uint32_t, uint8_t**, uint32_t, uint32_t);

void ___rpsl_block_marker(uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t);

void ___rpsl_describe_handle(uint8_t*, uint32_t, uint32_t*, uint32_t);
uint32_t ___rpsl_create_resource(uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t);
void ___rpsl_name_resource(uint32_t, uint8_t*, uint32_t);
uint32_t ___rpsl_dxop_binary_i32(uint32_t, uint32_t, uint32_t);

// DXIL Intrinsics
// TODO: Generate from hctdb
enum DXILOpCode
{
    // Binary float
    FMax = 35,  // returns a if a >= b, else b
    FMin = 36,  // returns a if a < b, else b

    // Binary int with two outputs
    IMul = 41,  // multiply of 32-bit operands to produce the correct full 64-bit result.

    // Binary int
    IMax = 37,  // IMax(a,b) returns a if a > b, else b
    IMin = 38,  // IMin(a,b) returns a if a < b, else b

    // Binary uint with carry or borrow
    UAddc = 44,  // unsigned add of 32-bit operand with the carry
    USubb = 45,  // unsigned subtract of 32-bit operands with the borrow

    // Binary uint with two outputs
    UDiv = 43,  // unsigned divide of the 32-bit operand src0 by the 32-bit operand src1.
    UMul = 42,  // multiply of 32-bit operands to produce the correct full 64-bit result.

    // Binary uint
    UMax = 39,  // unsigned integer maximum. UMax(a,b) = a > b ? a : b
    UMin = 40,  // unsigned integer minimum. UMin(a,b) = a < b ? a : b

    // Bitcasts with different sizes
    BitcastF16toI16 = 125,  // bitcast between different sizes
    BitcastF32toI32 = 127,  // bitcast between different sizes
    BitcastF64toI64 = 129,  // bitcast between different sizes
    BitcastI16toF16 = 124,  // bitcast between different sizes
    BitcastI32toF32 = 126,  // bitcast between different sizes
    BitcastI64toF64 = 128,  // bitcast between different sizes

    // Dot product with accumulate
    Dot2AddHalf     = 162,  // 2D half dot product with accumulate to float
    Dot4AddI8Packed = 163,  // signed dot product of 4 x i8 vectors packed into i32, with accumulate to i32
    Dot4AddU8Packed = 164,  // unsigned dot product of 4 x u8 vectors packed into i32, with accumulate to i32

    // Dot
    Dot2 = 54,  // Two-dimensional vector dot-product
    Dot3 = 55,  // Three-dimensional vector dot-product
    Dot4 = 56,  // Four-dimensional vector dot-product

    // Double precision
    LegacyDoubleToFloat  = 132,  // legacy fuction to convert double to float
    LegacyDoubleToSInt32 = 133,  // legacy fuction to convert double to int32
    LegacyDoubleToUInt32 = 134,  // legacy fuction to convert double to uint32
    MakeDouble           = 101,  // creates a double value
    SplitDouble          = 102,  // splits a double into low and high parts

    // Legacy floating-point
    LegacyF16ToF32 = 131,  // legacy fuction to convert half (f16) to float (f32) (this is not related to min-precision)
    LegacyF32ToF16 = 130,  // legacy fuction to convert float (f32) to half (f16) (this is not related to min-precision)

    // Packing intrinsics
    Pack4x8 = 220,  // packs vector of 4 signed or unsigned values into a packed datatype, drops or clamps unused bits

    // Quaternary
    Bfi = 53,  // Given a bit range from the LSB of a number, places that number of bits in another number at any offset

    // Tertiary float
    FMad = 46,  // floating point multiply & add
    Fma  = 47,  // fused multiply-add

    // Tertiary int
    IMad = 48,  // Signed integer multiply & add
    Ibfe = 51,  // Integer bitfield extract
    Msad = 50,  // masked Sum of Absolute Differences.

    // Tertiary uint
    UMad = 49,  // Unsigned integer multiply & add
    Ubfe = 52,  // Unsigned integer bitfield extract

    // Unary float - rounding
    Round_ne = 26,  // floating-point round to integral float.
    Round_ni = 27,  // floating-point round to integral float.
    Round_pi = 28,  // floating-point round to integral float.
    Round_z  = 29,  // floating-point round to integral float.

    // Unary float
    Acos =
        15,  // Returns the arccosine of the specified value. Input should be a floating-point value within the range of -1 to 1.
    Asin =
        16,  // Returns the arccosine of the specified value. Input should be a floating-point value within the range of -1 to 1
    Atan = 17,  // Returns the arctangent of the specified value. The return value is within the range of -PI/2 to PI/2.
    Cos  = 12,  // returns cosine(theta) for theta in radians.
    Exp  = 21,  // returns 2^exponent
    FAbs = 6,   // returns the absolute value of the input value.
    Frc  = 22,  // extract fracitonal component.
    Hcos = 18,  // returns the hyperbolic cosine of the specified value.
    Hsin = 19,  // returns the hyperbolic sine of the specified value.
    Htan = 20,  // returns the hyperbolic tangent of the specified value.
    IsFinite = 10,  // Returns true if x is finite, false otherwise.
    IsInf    = 9,   // Returns true if x is +INF or -INF, false otherwise.
    IsNaN    = 8,   // Returns true if x is NAN or QNAN, false otherwise.
    IsNormal = 11,  // returns IsNormal
    Log      = 23,  // returns log base 2.
    Rsqrt    = 25,  // returns reciprocal square root (1 / sqrt(src)
    Saturate = 7,   // clamps the result of a single or double precision floating point value to [0.0f...1.0f]
    Sin      = 13,  // returns sine(theta) for theta in radians.
    Sqrt     = 24,  // returns square root
    Tan      = 14,  // returns tan(theta) for theta in radians.

    // Unary int
    Bfrev     = 30,  // Reverses the order of the bits.
    Countbits = 31,  // Counts the number of bits in the input integer.
    FirstbitLo =
        32,  // Returns the location of the first set bit starting from the lowest order bit and working upward.
    FirstbitSHi = 34,  // Returns the location of the first set bit from the highest order bit based on the sign.

    // Unary uint
    FirstbitHi =
        33,  // Returns the location of the first set bit starting from the highest order bit and working downward.

    // Unpacking intrinsics
    Unpack4x8 = 219,  // unpacks 4 8-bit signed or unsigned values into int32 or int16 vector
};
