----------------------------------------------------------------------------------
-- Company: 
-- Engineer: Benno
-- 
-- Create Date:    14:43:04 11/11/2010 
-- Design Name: 
-- Module Name:    pc_uart_receive - Behavioral 
-- Project Name: 
-- Target Devices: 
-- Tool versions: 
-- Description: This is a simple receive-only-RS232 UART.
--					 Settings:	(8N1)
--									bit_rate: 50-500000 bit/s
--									parity: none
--									stop-bits: 1
--									Flow control: no
--									Data_bits: 8
--
-- Dependencies: 
--
-- Revision: 
-- Revision 0.01 - File Created
-- Revision 0.02 - Shift Register ausgetauscht - Florian Piazza
-- Additional Comments: 
--
----------------------------------------------------------------------------------
library IEEE;
use IEEE.STD_LOGIC_1164.all;
use IEEE.NUMERIC_STD.all;

-- Uncomment the following library declaration if using
-- arithmetic functions with Signed or Unsigned values
--use IEEE.NUMERIC_STD.ALL;

-- Uncomment the following library declaration if instantiating
-- any Xilinx primitives in this code.
--library UNISIM;
--use UNISIM.VComponents.all;

-- tag::interface[]
entity pc_uart_receive is
  port
  (
    clk        : in std_logic;
    reset      : in std_logic;
    rs232_bit  : in std_logic;
    data_out   : out std_logic_vector(7 downto 0);
    data_valid : out std_logic
  );
end pc_uart_receive;
-- end::interface[]

architecture Behavioral of pc_uart_receive is

  constant fpga_frequency : integer := 12000000;
  constant bit_rate       : integer := 9600;

  --  n x m bit shift register with rotation support
  component nxmBitShiftReg is
    generic
    (
      N : integer range 2 to 256 := 4; -- number of registers, IMPORTANT: N = 1 is not allowed (multiplexer config)
      M : integer range 1 to 256 := 2 -- bits per register
    );
    port
    (
      CLK    : in std_logic; -- input clock
      SR     : in std_logic; -- set/reset signal
      SRINIT : in std_logic_vector(n * m - 1 downto 0); -- init value for reset
      CE     : in std_logic; -- enable signal
      OPMODE : in std_logic_vector(1 downto 0); -- operation mode (see list of modes)
      DIN    : in std_logic_vector(m - 1 downto 0); -- input value when shifting
      DOUT   : out std_logic_vector(m - 1 downto 0); -- output: word shifted out resp. last word rotated
      DOUT_F : out std_logic_vector(n * m - 1 downto 0) -- output: complete register
    );
  end component nxmBitShiftReg;

  signal bit_cnt                       : unsigned(3 downto 0);
  signal bit_sampled                   : std_logic;
  signal bit_sampled_enable            : std_logic;
  signal clk_cnt                       : integer range 0 to 1048575;
  signal data_out_sr                   : std_logic_vector(8 downto 0);
  signal rs232_bit_syn, rs232_bit_last : std_logic;
  signal bit_receiving_time            : unsigned(19 downto 0);

  -- internal state machine states
  type state_type is (WAIT_DATA, SAMPLING, READY);
  signal state : state_type;

begin

  -- This shiftregister stores the transmitted 9 bits (s12345678)
  Inst_shiftregister : nxmBitShiftReg
  generic
  map (
  N => 9,
  M => 1
  )
  port map
  (
    clk    => clk,
    sr     => reset,
    srinit => (others => '0'),
    ce     => bit_sampled_enable,
    opmode => "00",
    din => (0 => bit_sampled),
    dout   => open,
    dout_f => data_out_sr
  );

  -- set duration of 1 bit in fpga-clks
  bit_receiving_time <= (to_unsigned(fpga_frequency/bit_rate, bit_receiving_time'length));

  -- route the shift register output to top level modul
  data_out <= data_out_sr(8 downto 1);
  RECEIVE : process (clk)
  begin
    if rising_edge(clk) then
      if (reset = '1') then
        state              <= WAIT_DATA;
        clk_cnt            <= 0;
        bit_cnt            <= (others => '0');
        data_valid         <= '0';
        bit_sampled        <= '0';
        bit_sampled_enable <= '0';
      else

        case state is

          when WAIT_DATA =>
            data_valid <= '0';
            -- a rs232_bit falling edge initiates sampling
            if ((rs232_bit_syn = '0') and (rs232_bit_last = '1')) then
              state <= SAMPLING;
            end if;

          when SAMPLING =>
            -- sample value at the middle bit position
            if ((clk_cnt = to_integer(unsigned(bit_receiving_time))/2)) then
              bit_sampled        <= rs232_bit;
              bit_sampled_enable <= '1';
            else
              bit_sampled_enable <= '0';
            end if;

            -- Clock counter
            if (clk_cnt < to_integer(bit_receiving_time)) then
              clk_cnt <= clk_cnt + 1;
            else
              clk_cnt <= 0;
              bit_cnt <= bit_cnt + 1;
            end if;

            -- After last bit is sampled, the state is switched to ready
            if (bit_cnt > 8) then
              state <= READY;
            end if;

          when READY =>
            -- In Ready-state, the byte is sent
            data_valid <= '1';
            bit_cnt    <= (others => '0');
            clk_cnt    <= 0;
            state      <= WAIT_DATA;

        end case;
      end if;
    end if;
  end process;

  -- This process buffers signals to realize edge detection
  BUFFERING : process (CLK)
  begin
    if rising_edge(CLK) then
      if RESET = '1' then
        rs232_bit_syn  <= '0';
        rs232_bit_last <= '0';
      else
        rs232_bit_syn  <= rs232_bit;
        rs232_bit_last <= rs232_bit_syn;
      end if;
    end if;
  end process;

end Behavioral;